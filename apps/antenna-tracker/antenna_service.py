#!/usr/bin/env python3
"""
Antenna service abstraction used by gps-ingest REPL.

Provides a uniform API over either a real ODrive-backed antenna or a mock
software implementation when hardware is not available.
"""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Dict, Tuple, Optional


@dataclass
class AntennaConfig:
    # Motion limits (degrees)
    limits_yaw_deg: Tuple[float, float] = (-370.0, 370.0)  # Multi-turn support to match manual.py
    limits_pitch_deg: Tuple[float, float] = (-1.0, 90.0)
    # Gear ratios for converting degrees <-> motor turns (optional if using real controller differently)
    gear_ratio_yaw: float = 30.0
    gear_ratio_pitch: float = 36.0
    # Hardware toggles
    use_real_hardware: bool = False
    # Optional debug prints
    debug: bool = False
    # ODrive motor/controller settings
    pole_pairs: int = 7
    torque_constant: float = 8.27 / 150.0
    current_limit: float = 30.0
    regen_current_limit: float = 4.0
    dc_max_negative_current: float = -6.0
    dc_max_positive_current: float = 60.0
    calibration_current: float = 8.0
    calibration_timeout: float = 30.0
    # Controller gains
    pos_gain: float = 2.0
    vel_gain: float = 0.10
    vel_integrator_gain: float = 3.0
    # Trap trajectory defaults (in degrees per second)
    trap_vel_limit_degps: float = 120.0
    trap_accel_limit_degps2: float = 90.0
    trap_decel_limit_degps2: float = 90.0
    # Direction (+1 normal, -1 if mechanics/wiring are reversed)
    dir_yaw: float = +1.0
    dir_pitch: float = +1.0
    # Velocity control (controller mode)
    vel_ramp_rate_degps2_yaw: float = 180.0
    vel_ramp_rate_degps2_pitch: float = 180.0
    max_vel_degps_yaw: float = 120.0
    max_vel_degps_pitch: float = 120.0


class AntennaService:
    """Facade for antenna control. Falls back to a mock implementation if hardware is not enabled."""

    def __init__(self, config: AntennaConfig):
        self.config = config
        self._connected = False
        self._configured = False
        self._calibrated = False
        self._engaged = False
        # Internal state (degrees)
        self._yaw_deg = 0.0
        self._pitch_deg = 0.0
        # Hardware handles (if any)
        self._device = None  # ODrive device
        self._axes: Dict[str, object] = {}  # {'yaw': axis0, 'pitch': axis1}
        self._offset_turns: Dict[str, float] = {"yaw": 0.0, "pitch": 0.0}
        # Lazily imported enums (only when hardware is enabled)
        self._AxisState = None
        self._ControlMode = None
        self._InputMode = None

    # ---- Lifecycle ----
    def connect(self) -> bool:
        if not self.config.use_real_hardware:
            self._connected = True
            return True
        try:
            import odrive  # type: ignore
            from odrive.enums import AxisState, ControlMode, InputMode  # type: ignore
            self._AxisState = AxisState
            self._ControlMode = ControlMode
            self._InputMode = InputMode
            print("AntennaService: Searching for ODrive...")
            self._device = odrive.find_any()
            if not self._device:
                print("AntennaService: no ODrive found")
                self._connected = False
                return False
            self._axes = {"yaw": self._device.axis0, "pitch": self._device.axis1}
            self._device.clear_errors()
            self._connected = True
            if self.config.debug:
                print(f"AntennaService: connected to ODrive serial {self._device.serial_number}")
            return True
        except Exception as e:  # pragma: no cover
            print("AntennaService: hardware connect failed:", e)
            self._connected = False
            return False

    def configure(self) -> bool:
        if not self._connected:
            return False
        if not self.config.use_real_hardware:
            self._configured = True
            return True
        try:
            dev = self._device
            assert dev is not None
            # Global device configuration
            dev.config.enable_brake_resistor = True
            dev.config.brake_resistance = 2.0
            dev.config.dc_max_negative_current = self.config.dc_max_negative_current
            dev.config.dc_max_positive_current = self.config.dc_max_positive_current
            dev.config.max_regen_current = self.config.regen_current_limit
            # Per-axis configuration
            for name, axis in self._axes.items():
                # Mirror manual.py behavior
                axis.controller.config.vel_limit = float('inf')
                mcfg = axis.motor.config
                ccfg = axis.controller.config
                tcfg = axis.trap_traj.config
                # Motor settings
                mcfg.pole_pairs = self.config.pole_pairs
                mcfg.torque_constant = self.config.torque_constant
                mcfg.current_lim = self.config.current_limit
                mcfg.calibration_current = self.config.calibration_current
                # Controller settings
                ccfg.input_mode = self._InputMode.TRAP_TRAJ
                ccfg.control_mode = self._ControlMode.POSITION_CONTROL
                ccfg.pos_gain = self.config.pos_gain
                ccfg.vel_gain = self.config.vel_gain
                ccfg.vel_integrator_gain = self.config.vel_integrator_gain
                # Trap trajectory limits (in motor turns per second)
                if name == 'yaw':
                    gear_ratio = self.config.gear_ratio_yaw
                else:  # pitch
                    gear_ratio = self.config.gear_ratio_pitch

                tcfg.vel_limit = (self.config.trap_vel_limit_degps / 360.0) * gear_ratio
                tcfg.accel_limit = (self.config.trap_accel_limit_degps2 / 360.0) * gear_ratio
                tcfg.decel_limit = (self.config.trap_decel_limit_degps2 / 360.0) * gear_ratio
                # Keep controller vel_limit in sync with trap traj
                ccfg.vel_limit = tcfg.vel_limit
                # Ensure non-circular setpoints for both axes
                ccfg.circular_setpoints = False
            self._configured = True
            if self.config.debug:
                print("AntennaService: configured motors and controller")
            return True
        except Exception as e:
            print("AntennaService: configure failed:", e)
            return False

    def calibrate(self) -> bool:
        if not self._configured:
            return False
        if not self.config.use_real_hardware:
            self._calibrated = True
            return True
        try:
            dev = self._device
            assert dev is not None
            dev.clear_errors()
            timeout_s = self.config.calibration_timeout
            # Start calibration on both axes simultaneously
            for name, ax in self._axes.items():
                if self.config.debug:
                    print(f"AntennaService: requesting calibration for {name} axis...")
                ax.requested_state = self._AxisState.FULL_CALIBRATION_SEQUENCE

            # Wait until both axes are idle or timeout
            start = time.time()
            while True:
                all_idle = all(ax.current_state == self._AxisState.IDLE for ax in self._axes.values())
                if all_idle:
                    break
                if time.time() - start > timeout_s:
                    print("AntennaService: calibration timed out for one or more axes")
                    return False
                time.sleep(0.2)

            # Set both axes to closed loop control
            for name, ax in self._axes.items():
                ax.requested_state = self._AxisState.CLOSED_LOOP_CONTROL
            time.sleep(0.5)

            # Check for errors on both axes
            for name, ax in self._axes.items():
                if ax.error != 0:
                    try:
                        from odrive.utils import dump_errors  # type: ignore
                        dump_errors(dev)
                    except Exception:
                        pass
                    print(f"AntennaService: {name} axis error after calibration")
                    return False
                if self.config.debug:
                    print(f"AntennaService: {name} axis calibration complete")
            self._calibrated = True
            self._engaged = True
            if self.config.debug:
                print("AntennaService: calibration successful")
            return True
        except Exception as e:
            print("AntennaService: calibrate failed:", e)
            return False

    def engage(self) -> bool:
        if not self._calibrated:
            return False
        if not self.config.use_real_hardware:
            self._engaged = True
            return True
        try:
            for ax in self._axes.values():
                ax.requested_state = self._AxisState.CLOSED_LOOP_CONTROL
            time.sleep(0.2)
            self._engaged = True
            return not self._has_errors()
        except Exception:
            return False

    def disengage(self) -> bool:
        if not self.config.use_real_hardware:
            self._engaged = False
            return True
        try:
            for ax in self._axes.values():
                ax.requested_state = self._AxisState.IDLE
            time.sleep(0.2)
            self._engaged = False
            return not self._has_errors()
        except Exception:
            return False

    def stop(self) -> bool:
        if not self.config.use_real_hardware:
            return True
        try:
            for ax in self._axes.values():
                ax.controller.input_vel = 0
            return not self._has_errors()
        except Exception:
            return False

    # ---- Velocity control (controller mode) ----
    def enter_velocity_mode(self) -> bool:
        if not self._connected:
            return False
        if not self.config.use_real_hardware:
            # No-op in mock mode
            return True
        try:
            for name, ax in self._axes.items():
                ccfg = ax.controller.config
                ccfg.control_mode = self._ControlMode.VELOCITY_CONTROL
                ccfg.input_mode = self._InputMode.VEL_RAMP
                # Configure ramp rates per axis
                if name == 'yaw':
                    ramp_deg_s2 = self.config.vel_ramp_rate_degps2_yaw
                    max_deg_s = self.config.max_vel_degps_yaw
                    gear = self.config.gear_ratio_yaw
                else:
                    ramp_deg_s2 = self.config.vel_ramp_rate_degps2_pitch
                    max_deg_s = self.config.max_vel_degps_pitch
                    gear = self.config.gear_ratio_pitch
                ccfg.vel_ramp_rate = (ramp_deg_s2 / 360.0) * gear
                ccfg.vel_limit = max(1.0, (max_deg_s / 360.0) * gear * 1.5)
            return True
        except Exception:
            return False

    def set_velocity(self, *, pitch_dps: Optional[float] = None, yaw_dps: Optional[float] = None) -> bool:
        if pitch_dps is None and yaw_dps is None:
            return True
        if not self.config.use_real_hardware:
            # In mock mode, just accept
            return True
        try:
            if pitch_dps is not None:
                turns_per_s = (self.config.dir_pitch * pitch_dps / 360.0) * self.config.gear_ratio_pitch
                self._axes['pitch'].controller.input_vel = turns_per_s
            if yaw_dps is not None:
                turns_per_s = (self.config.dir_yaw * yaw_dps / 360.0) * self.config.gear_ratio_yaw
                self._axes['yaw'].controller.input_vel = turns_per_s
            return not self._has_errors()
        except Exception:
            return False

    def calculate_max_permissible_velocity(self, axis_name: str, requested_velocity_degps: float) -> float:
        """Limit velocity so motion can stop before hitting position limits."""
        if requested_velocity_degps == 0.0:
            return 0.0
        pos = self.get_position()
        current_pos = pos[axis_name]
        lo, hi = (self.config.limits_pitch_deg if axis_name == 'pitch' else self.config.limits_yaw_deg)
        max_accel = self.config.vel_ramp_rate_degps2_pitch if axis_name == 'pitch' else self.config.vel_ramp_rate_degps2_yaw
        if requested_velocity_degps > 0:
            available = hi - current_pos
            direction = 1.0
        else:
            available = current_pos - lo
            direction = -1.0
        if available <= 0:
            return 0.0
        max_stop_vel = (2.0 * max_accel * max(available, 0.0)) ** 0.5
        limited = max_stop_vel if abs(requested_velocity_degps) > max_stop_vel else abs(requested_velocity_degps)
        return direction * limited

    def zero(self) -> None:
        if not self.config.use_real_hardware:
            self._yaw_deg = 0.0
            self._pitch_deg = 0.0
            return
        for name, ax in self._axes.items():
            self._offset_turns[name] = float(ax.encoder.pos_estimate)

    # ---- Status ----
    def get_position(self) -> Dict[str, float]:
        if not self.config.use_real_hardware:
            return {"yaw": self._yaw_deg, "pitch": self._pitch_deg}
        pos: Dict[str, float] = {}
        for name, ax in self._axes.items():
            turns = float(ax.encoder.pos_estimate) - self._offset_turns[name]
            if name == 'yaw':
                deg = (turns / self.config.gear_ratio_yaw) * 360.0
                pos[name] = self.config.dir_yaw * deg
            else:
                deg = (turns / self.config.gear_ratio_pitch) * 360.0
                pos[name] = self.config.dir_pitch * deg
        return pos

    def status(self) -> Dict[str, bool]:
        return {
            "connected": self._connected,
            "configured": self._configured,
            "calibrated": self._calibrated,
            "engaged": self._engaged,
        }

    # ---- Command ----
    def set_position(self, *, pitch: Optional[float] = None, yaw: Optional[float] = None, bypass_limits: bool = False) -> bool:
        # Always print when called, with arguments
        if self.config.debug:
            print(f"AntennaService: set_position called with pitch={pitch} yaw={yaw} bypass_limits={bypass_limits}")

        if pitch is None and yaw is None:
            return True

        # Clamp in logical space (positive-up convention) with optional debug warnings
        orig_pitch, orig_yaw = pitch, yaw
        if pitch is not None and not bypass_limits:
            lo, hi = self.config.limits_pitch_deg
            new_pitch = max(lo, min(hi, pitch))
            if new_pitch != pitch and self.config.debug:
                print(f"WARNING: Pitch clamped {pitch:.2f}° -> {new_pitch:.2f}°")
            pitch = new_pitch
        if yaw is not None and not bypass_limits:
            lo, hi = self.config.limits_yaw_deg
            new_yaw = max(lo, min(hi, yaw))
            if new_yaw != yaw and self.config.debug:
                print(f"WARNING: Yaw clamped {yaw:.2f}° -> {new_yaw:.2f}°")
            yaw = new_yaw

        if not self.config.use_real_hardware:
            if self.config.debug:
                print(f"AntennaService: (TEST) set_position to pitch={pitch} yaw={yaw}")
            if pitch is not None:
                self._pitch_deg = pitch
            if yaw is not None:
                self._yaw_deg = yaw
            if self.config.debug:
                print(f"AntennaService: (TEST) internal state now pitch={self._pitch_deg:.2f} yaw={self._yaw_deg:.2f}")
            return True

        try:
            # Map to hardware direction and send
            if self.config.debug:
                print(f"AntennaService: (HW) set_position to pitch={pitch} yaw={yaw}")
            if pitch is not None:
                pitch_hw = self.config.dir_pitch * pitch
                turns = (pitch_hw / 360.0) * self.config.gear_ratio_pitch + self._offset_turns["pitch"]
                if self.config.debug:
                    print(f"AntennaService: (HW) pitch command: pitch_hw={pitch_hw:.2f}, turns={turns:.4f}")
                self._axes["pitch"].controller.input_pos = turns
            if yaw is not None:
                yaw_hw = self.config.dir_yaw * yaw
                turns = (yaw_hw / 360.0) * self.config.gear_ratio_yaw + self._offset_turns["yaw"]
                if self.config.debug:
                    print(f"AntennaService: (HW) yaw command: yaw_hw={yaw_hw:.2f}, turns={turns:.4f}")
                self._axes["yaw"].controller.input_pos = turns
            return not self._has_errors()
        except Exception as e:
            print("AntennaService: set_position failed:", e)
            return False

    # ---- Internal ----
    def _has_errors(self) -> bool:
        if not self.config.use_real_hardware:
            return False
        try:
            if any(getattr(ax, 'error', 0) for ax in self._axes.values()):
                try:
                    from odrive.utils import dump_errors  # type: ignore
                    dump_errors(self._device)
                except Exception:
                    pass
                return True
            return False
        except Exception:
            return True
