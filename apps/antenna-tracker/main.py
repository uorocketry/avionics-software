#!/usr/bin/env python3
"""
gps-ingest CLI REPL for antenna integration.

Commands:
  - gps:        stream-print latest GPS samples; Ctrl-C to stop
  - prediction: print current prediction, pointing angles and antenna rotation
  - rotate:     print the command that would rotate antenna to predicted angles
  - yolo:       continuously (auto) print predicted rotation; Ctrl-C to stop

This CLI uses PredictorService as the API layer managing background ingestion
and prediction.
"""

from __future__ import annotations

import sys
import time
from dataclasses import asdict
import os
import json

from predictor_service import PredictorConfig, PredictorService
from antenna_service import AntennaConfig, AntennaService
from remote import XboxController


def _print_gps(sample) -> None:
    print(
        f"[{sample.ts_iso}] lat={sample.lat:.8f} lon={sample.lon:.8f} alt={sample.alt_m:.1f}m src={sample.source}"
    )


def cmd_gps(svc: PredictorService) -> None:
    print("Streaming GPS (Ctrl-C to stop)...")
    try:
        # Print existing history first
        hist = svc.get_history()
        last_ts = None
        for s in hist:
            _print_gps(s)
            last_ts = s.ts_iso
        while True:
            s = svc.get_latest()
            if s and s.ts_iso != last_ts:
                _print_gps(s)
                last_ts = s.ts_iso
            time.sleep(0.1)
    except KeyboardInterrupt:
        print("\nStopped GPS stream.")


def cmd_prediction(svc: PredictorService) -> None:
    latest = svc.get_latest()
    pred = svc.get_prediction()
    if not latest:
        print("No GPS yet.")
        return
    print("Antenna:")
    print(
        f"  lat={svc.config.antenna_lat_deg:.8f} lon={svc.config.antenna_lon_deg:.8f} alt={svc.config.antenna_alt_m:.1f}m"
    )
    print("Latest GPS:")
    _print_gps(latest)
    if not pred:
        print("No prediction yet.")
        return
    az, el = svc.compute_az_el(pred.lat, pred.lon, pred.alt_m)
    bearing = svc.initial_bearing_deg(
        svc.config.antenna_lat_deg, svc.config.antenna_lon_deg, pred.lat, pred.lon
    )

    # Preview the rotation we would send (without updating the accumulator)
    if svc.config.yaw_normalization == "continuous" and svc._last_yaw_cmd is not None:
        rotation_preview = svc.rotation_from_yaw_zero_continuous(az, svc._last_yaw_cmd)
    else:
        rotation_preview = svc._rotation_from_yaw_zero_basic(az)

    print("Prediction (+1s):")
    print(
        f"  lat={pred.lat:.8f} lon={pred.lon:.8f} alt={pred.alt_m:.1f}m conf={pred.confidence:.2f} ts={pred.ts_iso}"
    )
    print(f"  yaw(az from North)={az:.2f}° pitch(el)={el:.2f}°")
    print(f"  bearing={bearing:.2f}° from North; rotation(yaw-zero={svc.config.antenna_yaw_zero_deg:.1f}, mode={svc.config.yaw_normalization})={rotation_preview:.2f}°")

    # Show what would be sent to the antenna
    yaw_cmd, pit_cmd = svc.yaw_pitch_from_target(pred.lat, pred.lon, pred.alt_m)
    print(f"ROTATE pitch={pit_cmd:.2f}° yaw={yaw_cmd:.2f}°")


def cmd_rotate(svc: PredictorService, ant: AntennaService) -> None:
    pred = svc.get_prediction()
    if not pred:
        print("No prediction available.")
        return

    yaw_cmd, pit_cmd = svc.yaw_pitch_from_target(pred.lat, pred.lon, pred.alt_m)
    ok = ant.set_position(pitch=pit_cmd, yaw=yaw_cmd, bypass_limits=False)
    if ok:
        print(f"ROTATE pitch={pit_cmd:.2f}° yaw={yaw_cmd:.2f}° (sent)")
    else:
        print("Failed to send rotation command")


# def cmd_yolo(svc: PredictorService, ant: AntennaService) -> None:
def cmd_yolo(svc: PredictorService, ant: AntennaService | None = None, *, do_move: bool = False) -> None:
    print("Auto-tracking predictions (Ctrl-C to stop)...")
    try:
        while True:
            pred = svc.get_prediction()
            if pred:
                yaw_cmd, pit_cmd = svc.yaw_pitch_from_target(pred.lat, pred.lon, pred.alt_m)
                if do_move and ant is not None:
                    ant.set_position(pitch=pit_cmd, yaw=yaw_cmd, bypass_limits=False)
                print(f"\rpitch={pit_cmd:+06.2f}°  yaw={yaw_cmd:+07.2f}°", end="")
            time.sleep(0.3)
    except KeyboardInterrupt:
        print("\nStopped auto-tracking.")


def cmd_demo(svc: PredictorService, ant: AntennaService, jsonl_path: str, *, speed: float = 10.0, delay: float = 5.0, do_move: bool = False) -> None:
    """Preview first 5 lines of the JSONL, then stream using the replay script with delay."""
    abs_jsonl = os.path.abspath(jsonl_path)
    if not os.path.exists(abs_jsonl):
        print(f"JSONL not found: {abs_jsonl}")
        return
    print(f"Demo preview (first 5 lines) from {abs_jsonl}:")
    try:
        with open(abs_jsonl, 'r') as f:
            shown = 0
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                except Exception:
                    continue
                ts = obj.get('ts')
                lat = obj.get('lat')
                lon = obj.get('lon')
                alt = obj.get('altitude_m')
                src = obj.get('source', 'jsonl')
                print(f"  [{ts}] lat={lat:.8f} lon={lon:.8f} alt={alt:.1f}m src={src}")
                shown += 1
                if shown >= 5:
                    break
    except Exception as e:
        print("Failed to preview JSONL:", e)
        return

    # Reconfigure predictor to use external feed via replay script
    svc.stop()
    script_path = os.path.abspath(os.path.join(os.path.dirname(__file__), 'fake_gps_ingest.py'))
    svc.config.use_real_gps_ingest = True
    svc.config.num_samples = 8
    svc.config.gps_ingest_cmd = f"python3 {script_path} {abs_jsonl} --speed {speed} --delay {delay}"
    print(f"Starting demo stream with delay={delay}s, speed={speed}x...{' (MOVING)' if do_move else ''}")
    svc.start()
    # Run like yolo (auto-tracking printout); Ctrl-C to stop
    cmd_yolo(svc, ant if do_move else None, do_move=do_move)
def cmd_calibrate(ant: AntennaService) -> None:
    """Calibrate the antenna motors using the same logic as manual.py"""
    print("Clearing errors and calibrating axes...")
    if not ant._connected:
        print("ERROR: Antenna not connected. Run connect first.")
        return

    # Clear any existing errors before calibration
    if hasattr(ant, '_device') and ant._device:
        ant._device.clear_errors()

    print(f"Calibrating both axes simultaneously (timeout: {ant.config.calibration_timeout}s)...")

    # Start calibration on both axes
    for name, ax in ant._axes.items():
        print(f"Requesting calibration for {name} axis...")
        ax.requested_state = ant._AxisState.FULL_CALIBRATION_SEQUENCE

    start = time.time()
    # Wait for both axes to finish calibration or timeout
    while True:
        all_idle = all(ax.current_state == ant._AxisState.IDLE for ax in ant._axes.values())
        if all_idle:
            break
        if time.time() - start > ant.config.calibration_timeout:
            print("ERROR: Calibration timed out for one or more axes")
            return
        time.sleep(0.2)

    # Set both axes to closed loop control
    for name, ax in ant._axes.items():
        ax.requested_state = ant._AxisState.CLOSED_LOOP_CONTROL
    time.sleep(0.5)

    # Check for errors on both axes
    for name, ax in ant._axes.items():
        if ax.error != 0:
            print(f"ERROR: {name} axis has error after calibration")
            # Use the existing _has_errors method which will dump errors
            ant._has_errors()
            return
        print(f"{name.capitalize()} axis calibration complete.")

    print("All axes calibrated successfully.")
    ant._calibrated = True
    ant._engaged = True


def cmd_engage(ant: AntennaService) -> None:
    """Engage the antenna motors for closed loop control"""
    if not ant._connected:
        print("ERROR: Antenna not connected. Run connect first.")
        return
    if not ant._calibrated:
        print("ERROR: Antenna not calibrated. Run calibrate first.")
        return
    if ant._engaged:
        print("Antenna is already engaged.")
        return

    print("Engaging antenna motors...")
    if ant.engage():
        print("Antenna motors engaged successfully.")
    else:
        print("ERROR: Failed to engage antenna motors.")


def cmd_disengage(ant: AntennaService) -> None:
    """Disengage the antenna motors and set to idle"""
    if not ant._connected:
        print("ERROR: Antenna not connected. Run connect first.")
        return
    if not ant._engaged:
        print("Antenna is already disengaged.")
        return

    print("Disengaging antenna motors...")
    if ant.disengage():
        print("Antenna motors disengaged successfully.")
    else:
        print("ERROR: Failed to disengage antenna motors.")


def cmd_help() -> None:
    print("Available commands:")
    print("  gps           - Stream GPS history and live updates (Ctrl-C to stop)")
    print("  prediction    - Show current prediction, pointing angles and antenna rotation")
    print("  rotate        - Rotate antenna to current prediction (respects limits)")
    print("  yolo          - Auto-track predictions continuously (no motion)")
    print("  yolo-move     - Auto-track and MOVE antenna continuously (Ctrl-C to stop)")
    print("  demo [path]   - Replay JSONL with 5s delay (like yolo; no motion); default is apps/antenna-tracker/rocket_gps.jsonl")
    print("  demo-move [path] - Replay JSONL with 5s delay and MOVE antenna continuously")
    print("  pos <pitch> <yaw> [bypass] - Set absolute position in degrees; add 'bypass' to ignore limits")
    print("  zero          - Set current encoder offsets as zero for both axes")
    print("  set-yaw-zero <deg> - Set predictor yaw-zero at runtime (no restart)")
    print("  controller    - Enter controller mode (Xbox): LX=yaw, RY=pitch; X=zero, Y=go to zero, B=idle")
    print("  calibrate     - Calibrate antenna motors (clears errors, runs full calibration)")
    print("  engage        - Engage antenna motors for closed loop control")
    print("  disengage     - Disengage antenna motors and set to idle")
    print("  help          - Show this help message")
    print("  quit/exit     - Exit the program")


def _run_controller_mode(ant: AntennaService) -> None:
    """Controller mode similar to manual.py: LX=yaw, RY=pitch; X=zero, Y=go to zero, B=idle."""
    ctl = XboxController(deadzone=0.1)
    if not ctl.start():
        print("ERROR: failed to start controller")
        return
    print("\nController mode (VELOCITY). Ctrl-C to exit.")
    print("X=set zero here, Y=go to zero, B=idle")
    try:
        # Switch to velocity mode
        if not ant.enter_velocity_mode():
            print("ERROR: couldn't enter velocity mode")
            return
        last_x = last_y = last_b = False
        while True:
            if not ctl.is_connected():
                print("Controller disconnected.")
                break
            xb = ctl.get_button('X')
            yb = ctl.get_button('Y')
            bb = ctl.get_button('B')

            if xb and not last_x:
                ant.zero()
                print("Zero position set at current location")

            if yb and not last_y:
                # Temporarily move to zero using position control
                ant.set_position(pitch=0.0, yaw=0.0, bypass_limits=False)
                time.sleep(0.5)

            if bb and not last_b:
                print("EMERGENCY STOP: setting IDLE")
                ant.disengage()
                break

            last_x, last_y, last_b = xb, yb, bb

            yaw_in = ctl.get_axis('LX')
            pitch_in = -ctl.get_axis('RY')
            yaw_dps = yaw_in * ant.config.max_vel_degps_yaw
            pitch_dps = pitch_in * ant.config.max_vel_degps_pitch
            # Apply kinematic limits based on remaining travel
            yaw_dps_limited = ant.calculate_max_permissible_velocity('yaw', yaw_dps)
            pitch_dps_limited = ant.calculate_max_permissible_velocity('pitch', pitch_dps)
            ant.set_velocity(pitch_dps=pitch_dps_limited, yaw_dps=yaw_dps_limited)

            # Optional: show position when moving
            if abs(yaw_in) > 0.05 or abs(pitch_in) > 0.05:
                pos_deg = ant.get_position()
                print(f"\rpitch={pos_deg['pitch']:+06.2f}°  yaw={pos_deg['yaw']:+07.2f}°", end="")
            time.sleep(0.02)
    except KeyboardInterrupt:
        print("\nExiting controller mode...")
    finally:
        ctl.stop()


def cmd_pos(ant: AntennaService, args: str) -> None:
    """Set absolute pitch/yaw in degrees, like manual.py set_pos.

    Usage: pos <pitch> <yaw> [bypass]
    """
    parts = args.split()
    if len(parts) < 2:
        print("Usage: pos <pitch> <yaw> [bypass]")
        return
    try:
        pitch = float(parts[0])
        yaw = float(parts[1])
    except ValueError:
        print("Usage: pos <pitch> <yaw> [bypass]")
        return
    bypass = False
    if len(parts) >= 3 and parts[2].lower() in ("bypass", "true", "yes", "on"):
        bypass = True
    if bypass:
        # Mirror manual.py: announce limits bypass explicitly
        print(f"LIMITS BYPASSED: Pitch set to {pitch:.2f}°; Yaw set to {yaw:.2f}°")
    ok = ant.set_position(pitch=pitch, yaw=yaw, bypass_limits=bypass)
    # If not bypassing, AntennaService will WARN on clamps when debug=True
    if ok:
        print(f"Pitch Degrees: {pitch:.2f}, Yaw Degrees: {yaw:.2f}")
    else:
        print("ERROR: Failed to send position command")


def main() -> None:
    cfg = PredictorConfig()
    svc = PredictorService(cfg)
    ant = AntennaService(AntennaConfig())
    ant.connect(); ant.configure(); ant.calibrate(); ant.engage()
    svc.start()
    print("gps-ingest REPL. Type 'help' for commands.")
    try:
        while True:
            try:
                cmd = input("> ").strip().lower()
            except EOFError:
                break
            if cmd in ("quit", "exit"):
                break
            if cmd == "help":
                cmd_help()
            elif cmd == "gps":
                cmd_gps(svc)
            elif cmd == "prediction":
                cmd_prediction(svc)
            elif cmd == "rotate":
                cmd_rotate(svc, ant)
            elif cmd == "yolo":
                cmd_yolo(svc, None, do_move=False)
            elif cmd == "yolo-move":
                cmd_yolo(svc, ant, do_move=True)
            elif cmd.startswith("demo"):
                parts = cmd.split(maxsplit=1)
                if len(parts) == 2 and parts[1]:
                    path = parts[1].strip()
                else:
                    path = os.path.abspath(os.path.join(os.path.dirname(__file__), 'rocket_gps.jsonl'))
                if cmd.startswith("demo-move"):
                    cmd_demo(svc, ant, path, do_move=True)
                else:
                    cmd_demo(svc, ant, path, do_move=False)
            elif cmd.startswith("pos"):
                # usage: pos <pitch> <yaw> [bypass]
                args = cmd[len("pos"):].strip()
                cmd_pos(ant, args)
            elif cmd == "zero":
                ant.zero()
                print("Zero position set from encoder.")
            elif cmd.startswith("set-yaw-zero"):
                parts = cmd.split()
                if len(parts) != 2:
                    print("Usage: set-yaw-zero <degrees>")
                else:
                    try:
                        new_zero = float(parts[1])
                        svc.config.antenna_yaw_zero_deg = new_zero
                        svc.reset_yaw_continuity()
                        print(f"Set yaw-zero to {new_zero:.2f}° and reset continuity.")
                    except ValueError:
                        print("Invalid number. Usage: set-yaw-zero <degrees>")
            elif cmd == "controller":
                _run_controller_mode(ant)
            elif cmd == "calibrate":
                cmd_calibrate(ant)
            elif cmd == "engage":
                cmd_engage(ant)
            elif cmd == "disengage":
                cmd_disengage(ant)
            elif cmd == "":
                continue
            else:
                print("Unknown command. Type 'help' for available commands.")
    finally:
        print("Shutting down...")
        svc.stop()
        # Ensure antenna is disengaged and set to idle before exit
        if ant._engaged:
            print("Disengaging antenna motors...")
            ant.disengage()
        print("Cleanup complete.")


if __name__ == "__main__":
    main()
