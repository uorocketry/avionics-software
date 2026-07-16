#!/usr/bin/env python3
"""
Simplified Manual ODrive Control with Xbox Controller
"""
import time
import readline
import odrive
from odrive.enums import AxisState, ControlMode, InputMode
from odrive.utils import dump_errors
from remote import XboxController

class ManualODriveControl:
    """Manage manual control of an ODrive device, including Xbox controller support."""
    def __init__(self):
        self.device = None
        self.axes = {}  # {'yaw': axis0, 'pitch': axis1}
        self.offsets = {'yaw': 0.0, 'pitch': 0.0}
        self.calibrated = False
        self.debug = False
        self.controller = None
        self.controller_active = False

        self.cfg = {
            'gear_ratios': {'yaw': 30, 'pitch': 36},
            'pole_pairs': 7,
            'torque_constant': 8.27 / 150,
            'current_limit': 30.0,
            'regen_current_limit': 4.0, # Amps
            'dc_max_negative_current': -6.0, # Amps
            'dc_max_positive_current': 60.0, # Amps
            'calibration_current': 5.0, # Amps
            'calibration_timeout': 30.0,
            'ctrl_speed_scale': 10, # How fast the controller affects the target position
            'speed_step': 1.5,
            'speed_mult_limits': (0.1, 2.0),
            'max_speeds_deg': {
                'yaw': 120.0,    # output deg/s at full stick
                'pitch': 120.0,  # can tune each axis separately
            },
            'ramp_deg_s2': {
                'yaw': 180.0,   # decel/accel slope (deg/s²) - 0.25s stop time from full speed
                'pitch': 180.0, # stop time = max_speed_deg / ramp_deg_s2 = 90°/s / 360°/s² = 0.25s
            },
            'max_acceleration': 180.0,  # deg/s² - maximum acceleration for safety
            'limits': {
                'yaw': (-370, 370),
                # 'yaw': (-80, 80),
                'pitch': (-10, 90),
            },
            'gains': {
                'pos': 2.0,
                'vel': 0.10,
                'vel_int': 3.0,
            }
        }

    def connect(self):
        print("Connecting to ODrive...")
        try:
            self.device = odrive.find_any()
            if not self.device:
                print("ERROR: No ODrive found.")
                return False
            self.axes = {'yaw': self.device.axis0, 'pitch': self.device.axis1}
            self.device.clear_errors()
            print(f"Connected: Serial {self.device.serial_number}")
            return True
        except Exception as e:
            print("ERROR: Could not connect", e)
            return False

    def configure(self):
        print("Configuring motors...")



        self.device.config.enable_brake_resistor = True
        self.device.config.brake_resistance = 2.0

        self.device.config.dc_max_negative_current = self.cfg['dc_max_negative_current']
        self.device.config.dc_max_positive_current = self.cfg['dc_max_positive_current']
        self.device.config.max_regen_current = self.cfg['regen_current_limit']
        for name, axis in self.axes.items():
            axis.controller.config.vel_limit = float('inf') #revs/sec (max safe limit)
            mcfg = axis.motor.config
            ccfg = axis.controller.config
            tcfg = axis.trap_traj.config
            # Motor settings
            mcfg.pole_pairs = self.cfg['pole_pairs']
            mcfg.torque_constant = self.cfg['torque_constant']
            mcfg.current_lim = self.cfg['current_limit']
            mcfg.calibration_current = self.cfg['calibration_current']
            # Controller settings
            ccfg.input_mode = InputMode.TRAP_TRAJ
            ccfg.control_mode = ControlMode.POSITION_CONTROL
            ccfg.pos_gain = self.cfg['gains']['pos']
            ccfg.vel_gain = self.cfg['gains']['vel']
            ccfg.vel_integrator_gain = self.cfg['gains']['vel_int']
            # Set appropriate velocity limit for position control (not mirroring trap trajectory)
            ccfg.vel_limit = float('inf')  # Allow full speed for position control
            # Disable circular setpoints for yaw to prevent wrapping
            if name == 'yaw':
                ccfg.circular_setpoints = False
        print("Configuration done.")
        return True

    def calibrate(self):
        print("Clearing errors and calibrating axes...")
        self.device.clear_errors()  # Clear any existing errors before calibration
        print(f"Calibrating both axes simultaneously (timeout: {self.cfg['calibration_timeout']}s)...")
        # Start calibration on both axes
        for name, ax in self.axes.items():
            print(f"Requesting calibration for {name} axis...")
            ax.requested_state = AxisState.FULL_CALIBRATION_SEQUENCE

        start = time.time()
        # Wait for both axes to finish calibration or timeout
        while True:
            all_idle = all(ax.current_state == AxisState.IDLE for ax in self.axes.values())
            if all_idle:
                break
            if time.time() - start > self.cfg['calibration_timeout']:
                print("ERROR: Calibration timed out for one or more axes")
                return False
            time.sleep(0.2)

        # Set both axes to closed loop control
        for name, ax in self.axes.items():
            ax.requested_state = AxisState.CLOSED_LOOP_CONTROL
        time.sleep(0.5)

        # Check for errors on both axes
        for name, ax in self.axes.items():
            if ax.error != 0:
                print(f"ERROR: {name} axis has error after calibration")
                dump_errors(self.device)
                return False
            print(f"{name.capitalize()} axis calibration complete.")

        self.calibrated = True
        print("All axes calibrated.")
        return not self.has_errors()

    def has_errors(self):
        if any(ax.error for ax in self.axes.values()):
            dump_errors(self.device)
            return True
        return False

    def get_pos(self):
        pos = {}
        for name, axis in self.axes.items():
            turns = axis.encoder.pos_estimate - self.offsets[name]
            pos[name] = (turns / self.cfg['gear_ratios'][name]) * 360.0
        return pos

    def set_pos(self, pitch=None, yaw=None, verbose=True, bypass_limits=False):
        for name, deg in [('pitch', pitch), ('yaw', yaw)]:
            if deg is None:
                continue
            if bypass_limits:
                clamped = deg
                if verbose:
                    print(f"LIMITS BYPASSED: {name.capitalize()} set to {deg}°")
            else:
                lo, hi = self.cfg['limits'][name]
                clamped = max(lo, min(hi, deg))
                if verbose and clamped != deg:
                    print(f"WARNING: {name.capitalize()} clamped to {clamped}°")
            turns = (clamped / 360.0) * self.cfg['gear_ratios'][name] + self.offsets[name]
            self.axes[name].controller.input_pos = turns

        print(f"Pitch Degrees: {pitch}, Yaw Degrees: {yaw}")
        return not self.has_errors()

    def set_vel(self, pitch=None, yaw=None):
        for name, dps in [('pitch', pitch), ('yaw', yaw)]:
            if dps is None:
                continue
            turns_per_s = (dps / 360.0) * self.cfg['gear_ratios'][name]
            self.axes[name].controller.input_vel = turns_per_s

    def calculate_max_permissible_velocity(self, axis_name, requested_velocity):
        """
        Calculate maximum permissible velocity based on kinematic constraints.

        Args:
            axis_name: 'pitch' or 'yaw'
            requested_velocity: requested velocity in deg/s (can be negative)

        Returns:
            float: maximum permissible velocity in deg/s
        """
        if requested_velocity == 0:
            return 0.0

        # Get current position and limits
        current_pos = self.get_pos()[axis_name]
        limits = self.cfg['limits'][axis_name]
        max_accel = self.cfg.get('max_acceleration', 20.0)  # deg/s²

        # Determine direction and available travel distance
        if requested_velocity > 0:  # Moving in positive direction
            available_distance = limits[1] - current_pos  # Distance to upper limit
            direction = 1
        else:  # Moving in negative direction
            available_distance = current_pos - limits[0]  # Distance to lower limit
            direction = -1

        # If we're already at a limit, no movement allowed
        if available_distance <= 0:
            return 0.0

        # Kinematic equation: v² = v₀² + 2aΔx
        # For stopping from velocity v to 0: 0 = v² + 2(-a)Δx
        # Therefore: v = √(2aΔx)
        # Where: a = deceleration, Δx = available distance

        # Convert max_accel from deg/s² to deg/s² (already in correct units)
        max_velocity = (2 * max_accel * available_distance) ** 0.5

        # Apply direction
        max_velocity *= direction

        # Limit to the requested velocity magnitude
        if abs(requested_velocity) <= abs(max_velocity):
            return requested_velocity
        else:
            return max_velocity

    def stop(self):
        for ax in self.axes.values():
            ax.controller.input_vel = 0
        print("Motors stopped.")

    def zero(self):
        for name, ax in self.axes.items():
            self.offsets[name] = ax.encoder.pos_estimate
        print("Zero position set.")

    def status(self):
        pos = self.get_pos()
        print(f"Pitch: {pos['pitch']:.2f}°, Yaw: {pos['yaw']:.2f}°")
        for name, ax in self.axes.items():
            print(f"{name.capitalize()} state: {AxisState(ax.current_state).name}")
        print(f"Calibrated: {self.calibrated}")
        self.has_errors()

    def disengage(self):
        """Set both axes to IDLE state (disengaged)"""
        print("Disengaging motors...")
        for name, ax in self.axes.items():
            ax.requested_state = AxisState.IDLE
        time.sleep(0.5)  # Allow time for state change
        print("Motors disengaged.")
        return not self.has_errors()

    def engage(self):
        """Set both axes to CLOSED_LOOP_CONTROL state (engaged)"""
        if not self.calibrated:
            print("ERROR: Cannot engage - motors not calibrated. Run calibration first.")
            return False
        print("Engaging motors...")
        for name, ax in self.axes.items():
            ax.requested_state = AxisState.CLOSED_LOOP_CONTROL
        time.sleep(0.5)  # Allow time for state change
        print("Motors engaged.")
        return not self.has_errors()

    def toggle_debug(self, on):
        self.debug = on
        print("Debug", "ON" if on else "OFF")

    def _start_controller_services(self):
        if self.controller_active:
            return True
        try:
            self.controller = XboxController(deadzone=0.1)
            if not self.controller.start():
                print("ERROR: Failed to start Xbox controller.")
                return False
            self.controller_active = True
            return True
        except Exception as e:
            print("ERROR:", e)
            return False

    def _stop_controller_services(self):
        if not self.controller_active:
            return True
        print("Stopping controller...")
        if self.controller:
            self.controller.stop()
        self.controller_active = False
        self.stop()
        print("Xbox controller stopped.")
        return True

    def run_controller_loop(self):
        if not self._start_controller_services():
            return
        print("\nController mode (VELOCITY). Ctrl-C to exit.")
        print("X=set zero here, Y=return to zero, B=emergency stop")

        # Put both axes in velocity control mode with ramped input
        for name, ax in self.axes.items():
            ccfg = ax.controller.config
            ccfg.control_mode = ControlMode.VELOCITY_CONTROL
            ccfg.input_mode = InputMode.VEL_RAMP

            # Convert output ramp (deg/s²) -> motor turns/s²
            ramp_turns_s2 = (self.cfg['ramp_deg_s2'][name] / 360.0) * self.cfg['gear_ratios'][name]
            ccfg.vel_ramp_rate = ramp_turns_s2

            # Compute expected motor turns/s at full stick for this axis
            max_turns_per_s = (self.cfg['max_speeds_deg'][name] / 360.0) * self.cfg['gear_ratios'][name]

            # Give some headroom (50%) to avoid nuisance trips from measurement spikes
            ccfg.vel_limit = max(1.0, max_turns_per_s * 1.5)

            # Optional: relax overspeed tolerance a bit (default is ~1.2)
            try:
                ccfg.vel_limit_tolerance = 1.5
            except AttributeError:
                pass  # older firmware may not expose this

        # Button state tracking
        last_x = last_y = last_b = False



        try:
            while self.controller_active:
                if not self.controller.is_connected():
                    print("Controller disconnected.")
                    break

                # Check buttons
                xb = self.controller.get_button('X')
                yb = self.controller.get_button('Y')
                bb = self.controller.get_button('B')



                # X button: set zero at current position
                if xb and not last_x:
                    self.zero()
                    print("Zero position set at current location")

                # Y button: return to zero position
                if yb and not last_y:
                    print("Returning to zero position...")
                    # Switch to position control temporarily
                    for ax in self.axes.values():
                        ax.controller.config.control_mode = ControlMode.POSITION_CONTROL
                        ax.controller.config.input_mode = InputMode.TRAP_TRAJ

                    # Move to zero
                    self.set_pos(0.0, 0.0, bypass_limits=False)

                    # Wait for movement to complete
                    time.sleep(2.0)

                    # Switch back to velocity control with ramped input
                    for name, ax in self.axes.items():
                        ccfg = ax.controller.config
                        ccfg.control_mode = ControlMode.VELOCITY_CONTROL
                        ccfg.input_mode = InputMode.VEL_RAMP

                        # Recompute ramp rate and velocity limits for this axis
                        ramp_turns_s2 = (self.cfg['ramp_deg_s2'][name] / 360.0) * self.cfg['gear_ratios'][name]
                        ccfg.vel_ramp_rate = ramp_turns_s2

                        max_turns_per_s = (self.cfg['max_speeds_deg'][name] / 360.0) * self.cfg['gear_ratios'][name]
                        ccfg.vel_limit = max(1.0, max_turns_per_s * 1.5)

                    print("Returned to zero position")

                # B button: Emergency stop - set to idle and stop controller mode
                if bb and not last_b:
                    print("EMERGENCY STOP ACTIVATED!")
                    print("Setting motors to IDLE and stopping controller mode...")

                    # Stop all motors immediately
                    self.set_vel(0.0, 0.0)

                    # Set both axes to IDLE state (disengage)
                    for name, ax in self.axes.items():
                        ax.requested_state = AxisState.IDLE

                    # Stop controller services and exit loop
                    self._stop_controller_services()
                    return

                last_x, last_y, last_b = xb, yb, bb

                yaw_in   = self.controller.get_axis('LX')
                pitch_in = -self.controller.get_axis('RY')

                yaw_dps   = yaw_in   * self.cfg['max_speeds_deg']['yaw']
                pitch_dps = pitch_in * self.cfg['max_speeds_deg']['pitch']

                # Apply kinematic velocity limits
                yaw_dps_limited = self.calculate_max_permissible_velocity('yaw', yaw_dps)
                pitch_dps_limited = self.calculate_max_permissible_velocity('pitch', pitch_dps)
                if abs(yaw_in) > 0.05 or abs(pitch_in) > 0.05:  # Only print when moving
                    print(f"Limited velocities: Pitch {pitch_dps_limited:.1f}°/s, Yaw {yaw_dps_limited:.1f}°/s")

                self.set_vel(pitch_dps_limited, yaw_dps_limited)
                self.has_errors()

                # Print estimated position in degrees
                if abs(yaw_in) > 0.05 or abs(pitch_in) > 0.05:  # Only print when moving
                    pos_deg = self.get_pos()
                    print(f"Position: Pitch {pos_deg['pitch']:.2f}°, Yaw {pos_deg['yaw']:.2f}° | Velocity: Pitch {pitch_dps:.1f}°/s, Yaw {yaw_dps:.1f}°/s")

                time.sleep(0.02)
        except KeyboardInterrupt:
            print("\nCtrl-C detected. Setting motors to idle and stopping...")
            # Stop all motors immediately
            self.set_vel(0.0, 0.0)

            # Set both axes to IDLE state (disengage)
            for name, ax in self.axes.items():
                ax.requested_state = AxisState.IDLE

            print("Motors set to IDLE state.")
        finally:
            self.set_vel(0.0, 0.0)
            self._stop_controller_services()


def main():
    ctl = ManualODriveControl()
    if not (ctl.connect() and ctl.configure() and ctl.calibrate()):
        return
    cmds = ['help','status','zero','stop','pos','debug','controller','engage','disengage','calibrate','quit']
    readline.parse_and_bind("tab: complete")
    readline.set_completer(lambda t,s: [c for c in cmds if c.startswith(t)][s] if s < len(cmds) else None)
    print("Ready. Type 'help'.")
    while True:
        try:
            inp = input('> ').split()
            if not inp: continue
            cmd, *args = inp
            if cmd in ('quit','exit'):
                break
            if cmd == 'help':
                print("Commands: status, zero, stop, pos <pitch> <yaw>, debug <on|off>, controller, engage, disengage, calibrate, quit")
                print("Controller: X=zero here, Y=go to zero, B=emergency stop")
            elif cmd == 'status': ctl.status()
            elif cmd == 'zero': ctl.zero()
            elif cmd == 'stop': ctl.stop()
            elif cmd == 'pos' and len(args)==2: ctl.set_pos(float(args[0]), float(args[1]), bypass_limits=False)
            elif cmd == 'debug' and args: ctl.toggle_debug(args[0]=='on')
            elif cmd == 'controller': ctl.run_controller_loop()
            elif cmd == 'engage': ctl.engage()
            elif cmd == 'disengage': ctl.disengage()
            elif cmd == 'calibrate': ctl.calibrate()
            else:
                print("Unknown command")
        except KeyboardInterrupt:
            break

if __name__ == '__main__':
    main()
