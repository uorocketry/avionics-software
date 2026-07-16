#!/usr/bin/env python3
"""
A typed, callback-based reader for SBG devices that uses the sbgBasicLogger
executable to stream data.

This module provides a clean, modern, and easy-to-use Python interface for an
SBG device, abstracting away the subprocess and data parsing complexities.
"""

import os
import subprocess
import threading
import time
from dataclasses import dataclass
from typing import Optional, Callable, Dict, List, Literal, overload, Any, Set

# --- Data Models ---

@dataclass
class SbgEkfEulerData:
    """
    Represents the EKF Euler Angles data log.
    This log provides the roll, pitch, and yaw angles, along with their standard deviations.
    """
    status: int                 # EKF status information.
    roll: float                 # Roll angle. Unit: deg
    pitch: float                # Pitch angle. Unit: deg
    yaw: float                  # Yaw angle (heading). Unit: deg
    roll_std: float             # Standard deviation of the roll angle. Unit: deg
    pitch_std: float            # Standard deviation of the pitch angle. Unit: deg
    yaw_std: float              # Standard deviation of the yaw angle. Unit: deg
    mag_heading: float          # The magnetic heading. Unit: deg
    mag_declination: float      # Magnetic declination. Unit: deg
    mag_inclination: float      # Magnetic inclination. Unit: deg

@dataclass
class SbgEkfQuatData:
    """
    Represents the EKF Quaternion data log.
    This log provides the orientation as a quaternion, which avoids gimbal lock issues.
    """
    status: int                 # EKF status information.
    qW: float                   # Quaternion W component.
    qX: float                   # Quaternion X component.
    qY: float                   # Quaternion Y component.
    qZ: float                   # Quaternion Z component.
    roll_std: float             # Standard deviation of the roll angle. Unit: deg
    pitch_std: float            # Standard deviation of the pitch angle. Unit: deg
    yaw_std: float              # Standard deviation of the yaw angle. Unit: deg
    mag_declination: float      # Magnetic declination. Unit: deg
    mag_inclination: float      # Magnetic inclination. Unit: deg

@dataclass
class SbgEkfNavData:
    """
    Represents the EKF Navigation data log.
    This log provides the device's position (latitude, longitude, altitude) and
    velocities in the North-East-Down (NED) frame.
    """
    status: int                 # EKF status information.
    vel_n: float                # Velocity in the North direction. Unit: m/s
    vel_e: float                # Velocity in the East direction. Unit: m/s
    vel_d: float                # Velocity in the Down direction. Unit: m/s
    vel_n_std: float            # Standard deviation of the North velocity. Unit: m/s
    vel_e_std: float            # Standard deviation of the East velocity. Unit: m/s
    vel_d_std: float            # Standard deviation of the Down velocity. Unit: m/s
    latitude: float             # Latitude position. Unit: deg
    longitude: float            # Longitude position. Unit: deg
    altitude: float             # Altitude above mean sea level. Unit: m
    lat_std: float              # Standard deviation of the latitude. Unit: m
    lon_std: float              # Standard deviation of the longitude. Unit: m
    alt_std: float              # Standard deviation of the altitude. Unit: m
    undulation: float           # Geoid undulation. Unit: m

@dataclass
class SbgAirData:
    """
    Represents the Air Data log (e.g., from a Pitot tube or barometer).
    Contains barometric pressure, altitude, and airspeed information.
    """
    status: int                 # Air data sensor status.
    pressure: float             # Barometric pressure. Unit: hPa
    altitude: float             # Barometric altitude. Unit: m
    temperature: float          # Ambient temperature. Unit: °C
    air_speed: float            # True airspeed. Unit: m/s
    air_speed_std: float        # Standard deviation of the true airspeed. Unit: m/s

@dataclass
class SbgImuData:
    """
    Represents the IMU (Inertial Measurement Unit) data log.
    This log contains calibrated sensor data from the accelerometers, gyroscopes,
    and the device's internal temperature sensor.
    """
    status: int                 # IMU status information.
    accel_x: float              # Calibrated accelerometer X-axis. Unit: m/s^2
    accel_y: float              # Calibrated accelerometer Y-axis. Unit: m/s^2
    accel_z: float              # Calibrated accelerometer Z-axis. Unit: m/s^2
    gyro_x: float               # Calibrated gyroscope X-axis. Unit: deg/s
    gyro_y: float               # Calibrated gyroscope Y-axis. Unit: deg/s
    gyro_z: float               # Calibrated gyroscope Z-axis. Unit: deg/s
    temperature: float          # Internal IMU temperature. Unit: °C

@dataclass
class SbgGnssVelData:
    """
    Represents the GNSS Velocity data log.
    Contains velocity information calculated from the GNSS receiver.
    """
    status: int                 # GNSS velocity status.
    gps_time_ms: int            # GPS time of week. Unit: ms
    vel_n: float                # Velocity in the North direction. Unit: m/s
    vel_e: float                # Velocity in the East direction. Unit: m/s
    vel_d: float                # Velocity in the Down direction. Unit: m/s
    vel_std_n: float            # Standard deviation of the North velocity. Unit: m/s
    vel_std_e: float            # Standard deviation of the East velocity. Unit: m/s
    vel_std_d: float            # Standard deviation of the Down velocity. Unit: m/s
    course: float               # True heading from the GNSS. Unit: deg
    course_std: float           # Standard deviation of the course. Unit: deg

@dataclass
class SbgGnssPosData:
    """
    Represents the GNSS Position data log.
    Contains position information from the GNSS receiver.
    """
    status: int                 # GNSS position status.
    status_ext: int             # Extended GNSS position status.
    gps_time_ms: int            # GPS time of week. Unit: ms
    latitude: float             # Latitude position. Unit: deg
    longitude: float            # Longitude position. Unit: deg
    altitude: float             # Altitude above mean sea level. Unit: m
    undulation: float           # Geoid undulation. Unit: m
    lat_std: float              # Standard deviation of the latitude. Unit: m
    lon_std: float              # Standard deviation of the longitude. Unit: m
    alt_std: float              # Standard deviation of the altitude. Unit: m
    satellites_tracked: int     # Number of satellites tracked.
    satellites_used: int        # Number of satellites used in the computation.
    base_station_id: int        # ID of the DGPS/RTK base station.
    differential_age: int       # Age of the differential corrections. Unit: 0.01s

@dataclass
class SbgStatusData:
    """
    Represents the main device status log.
    Provides high-level status information about the device's main components.
    """
    status: int                 # General status summary.
    main_power: int             # Main power supply status.
    imu_power: int              # IMU power status.
    gps_power: int              # GNSS receiver power status.
    settings: int               # Status of the device settings.

@dataclass
class SbgUtcTimeData:
    """
    Represents the UTC Time data log.
    Provides date and time information synchronized with UTC.
    """
    status: int                 # UTC time synchronization status.
    year: int                   # Year.
    month: int                  # Month.
    day: int                    # Day.
    hour: int                   # Hour.
    minute: int                 # Minute.
    second: int                 # Second.
    nanosecond: int             # Nanosecond part of the time.
    gps_time_of_week_s: int     # GPS time of week. Unit: s
    flags: int                  # UTC data flags.
    valid: int                  # Validity status.
    time_s: int                 # Time in seconds.

@dataclass
class SbgGnssHdtData:
    """
    Represents the GNSS True Heading data log.
    Provides heading information from a dual-antenna GNSS setup.
    """
    status: int                 # GNSS heading status.
    gps_time_ms: int            # GPS time of week. Unit: ms
    heading: float              # True heading. Unit: deg
    heading_std: float          # Standard deviation of the heading. Unit: deg
    pitch: float                # Pitch angle from dual-antenna GNSS. Unit: deg
    pitch_std: float            # Standard deviation of the pitch. Unit: deg
    baseline: float             # Length of the baseline between the two GNSS antennas. Unit: m
    satellites_tracked: int     # Number of satellites tracked.
    satellites_used: int        # Number of satellites used in the heading computation.

@dataclass
class SbgGnssSatData:
    """
    Represents the GNSS Satellite data log.
    Provides a simple count of the visible satellites.
    """
    satellites_count: int       # The number of visible satellites.

@dataclass
class SbgImuShortData:
    """
    Represents the Short IMU data log.
    Provides integrated delta velocity and delta angle values over a short period.
    """
    status: int                 # IMU status.
    delta_vel_x: float          # Integrated delta velocity X-axis. Unit: m/s
    delta_vel_y: float          # Integrated delta velocity Y-axis. Unit: m/s
    delta_vel_z: float          # Integrated delta velocity Z-axis. Unit: m/s
    delta_angle_x: float        # Integrated delta angle X-axis. Unit: deg
    delta_angle_y: float        # Integrated delta angle Y-axis. Unit: deg
    delta_angle_z: float        # Integrated delta angle Z-axis. Unit: deg
    temperature: float          # Internal IMU temperature. Unit: °C

@dataclass
class SbgImuFastData:
    """
    Represents the Fast IMU data log.
    Similar to the short IMU log but intended for higher frequency output.
    """
    status: int                 # IMU status.
    delta_vel_x: float          # Integrated delta velocity X-axis. Unit: m/s
    delta_vel_y: float          # Integrated delta velocity Y-axis. Unit: m/s
    delta_vel_z: float          # Integrated delta velocity Z-axis. Unit: m/s
    delta_angle_x: float        # Integrated delta angle X-axis. Unit: deg
    delta_angle_y: float        # Integrated delta angle Y-axis. Unit: deg
    delta_angle_z: float        # Integrated delta angle Z-axis. Unit: deg
    temperature: float          # Internal IMU temperature. Unit: °C

@dataclass
class SbgMagData:
    """
    Represents the Magnetometer data log.
    Contains calibrated data from the magnetometers.
    """
    status: int                 # Magnetometer status.
    mag_x: float                # Calibrated magnetometer X-axis. Unit: uT
    mag_y: float                # Calibrated magnetometer Y-axis. Unit: uT
    mag_z: float                # Calibrated magnetometer Z-axis. Unit: uT
    temperature: float          # Internal magnetometer temperature. Unit: °C

@dataclass
class SbgDiagData:
    """
    Represents the device's diagnostics log.
    Contains information about the device's internal health and performance.
    """
    status: int                 # General diagnostics status.
    main_loop_time: int         # Execution time of the main loop. Unit: us
    computation_load: float     # CPU computation load. Unit: %
    cpu_temperature: float      # CPU temperature. Unit: °C

@dataclass
class SbgVelBodyData:
    """
    Represents the Body Velocity data log.
    Contains velocities expressed in the vehicle's body frame (X, Y, Z).
    """
    status: int                 # Velocity status.
    vel_x: float                # Velocity along the X-axis of the body frame. Unit: m/s
    vel_y: float                # Velocity along the Y-axis of the body frame. Unit: m/s
    vel_z: float                # Velocity along the Z-axis of the body frame. Unit: m/s
    vel_std_x: float            # Standard deviation of the X-axis velocity. Unit: m/s
    vel_std_y: float            # Standard deviation of the Y-axis velocity. Unit: m/s
    vel_std_z: float            # Standard deviation of the Z-axis velocity. Unit: m/s

# --- Main Reader Class ---
class SBGReader:
    """
    Manages the sbgBasicLogger process and provides a typed API to access
    and subscribe to SBG device data streams.
    """

    def __init__(self) -> None:
        """
        Initializes the SBGReader, setting up internal state for managing
        the subprocess, subscriptions, and latest measurements.
        """
        self._process: Optional[subprocess.Popen] = None
        self._reader_thread: Optional[threading.Thread] = None
        self._running = False
        self._subscriptions: Dict[str, Set[Callable]] = {}
        self._latest_measurements: Dict[str, Any] = {}

    def start(self, device_path: str, baudrate: int = 115200) -> None:
        """
        Starts the sbgBasicLogger process and begins reading data.
        """
        if self._running:
            raise RuntimeError("SBGReader is already running.")

        # logger_path = os.path.join(os.path.dirname(__file__), "sbgECom", "tools", "sbgBasicLogger", "sbgBasicLogger")
        logger_path = "./sbgBasicLogger"
        if not os.path.exists(logger_path):
            raise FileNotFoundError(f"sbgBasicLogger not found at {logger_path}")

        cmd = [logger_path, "-s", device_path, "-r", str(baudrate), "-p", "-f", "hexadecimal"]

        self._process = subprocess.Popen(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1
        )

        time.sleep(1)
        if self._process.poll() is not None:
            err_output = self._process.stderr.read().strip() if self._process.stderr else ""
            raise RuntimeError(f"sbgBasicLogger failed to start: {err_output}")

        self._running = True
        self._reader_thread = threading.Thread(target=self._read_loop, daemon=True)
        self._reader_thread.start()

    def stop(self) -> None:
        """
        Stops the sbgBasicLogger process and the reader thread gracefully.
        """
        if not self._running:
            return

        self._running = False
        if self._process:
            self._process.terminate()
            try:
                self._process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self._process.kill()

        if self._reader_thread:
            self._reader_thread.join()

    @overload
    def get_latest(self, message_type: Literal['euler']) -> Optional[SbgEkfEulerData]: ...
    @overload
    def get_latest(self, message_type: Literal['quat']) -> Optional[SbgEkfQuatData]: ...
    @overload
    def get_latest(self, message_type: Literal['nav']) -> Optional[SbgEkfNavData]: ...
    @overload
    def get_latest(self, message_type: Literal['airData']) -> Optional[SbgAirData]: ...
    @overload
    def get_latest(self, message_type: Literal['imuData']) -> Optional[SbgImuData]: ...
    @overload
    def get_latest(self, message_type: Literal['gnss1Vel']) -> Optional[SbgGnssVelData]: ...
    @overload
    def get_latest(self, message_type: Literal['gnss1Pos']) -> Optional[SbgGnssPosData]: ...
    @overload
    def get_latest(self, message_type: Literal['status']) -> Optional[SbgStatusData]: ...
    @overload
    def get_latest(self, message_type: Literal['utcTime']) -> Optional[SbgUtcTimeData]: ...
    @overload
    def get_latest(self, message_type: Literal['gnss1Hdt']) -> Optional[SbgGnssHdtData]: ...
    @overload
    def get_latest(self, message_type: Literal['gnss1Sat']) -> Optional[SbgGnssSatData]: ...
    @overload
    def get_latest(self, message_type: Literal['imuShort']) -> Optional[SbgImuShortData]: ...
    @overload
    def get_latest(self, message_type: Literal['imuFast']) -> Optional[SbgImuFastData]: ...
    @overload
    def get_latest(self, message_type: Literal['mag']) -> Optional[SbgMagData]: ...
    @overload
    def get_latest(self, message_type: Literal['diag']) -> Optional[SbgDiagData]: ...
    @overload
    def get_latest(self, message_type: Literal['velBody']) -> Optional[SbgVelBodyData]: ...

    def get_latest(self, message_type: str) -> Optional[Any]:
        return self._latest_measurements.get(message_type)

    @overload
    def subscribe(self, message_type: Literal['euler'], callback: Callable[[SbgEkfEulerData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['quat'], callback: Callable[[SbgEkfQuatData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['nav'], callback: Callable[[SbgEkfNavData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['airData'], callback: Callable[[SbgAirData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['imuData'], callback: Callable[[SbgImuData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['gnss1Vel'], callback: Callable[[SbgGnssVelData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['gnss1Pos'], callback: Callable[[SbgGnssPosData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['status'], callback: Callable[[SbgStatusData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['utcTime'], callback: Callable[[SbgUtcTimeData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['gnss1Hdt'], callback: Callable[[SbgGnssHdtData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['gnss1Sat'], callback: Callable[[SbgGnssSatData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['imuShort'], callback: Callable[[SbgImuShortData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['imuFast'], callback: Callable[[SbgImuFastData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['mag'], callback: Callable[[SbgMagData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['diag'], callback: Callable[[SbgDiagData], None]) -> None: ...
    @overload
    def subscribe(self, message_type: Literal['velBody'], callback: Callable[[SbgVelBodyData], None]) -> None: ...

    def subscribe(self, message_type: str, callback: Callable) -> None:
        if message_type not in self._subscriptions:
            self._subscriptions[message_type] = set()
        self._subscriptions[message_type].add(callback)

    def unsubscribe(self, message_type: str, callback: Callable) -> None:
        if message_type in self._subscriptions:
            self._subscriptions[message_type].discard(callback)

    def _read_loop(self) -> None:
        if not self._process or not self._process.stdout:
            return

        for line in iter(self._process.stdout.readline, ""):
            if not self._running:
                break
            if line.strip():
                self._parse_line(line)

    def _parse_value(self, value: str) -> Any:
        try:
            if value.lower().startswith('0x'):
                return int(value, 16)
            if value.lower() == 'nan':
                return float('nan')
            return int(value)
        except ValueError:
            try:
                return float(value)
            except ValueError:
                return value

    def _parse_line(self, line: str) -> None:
        if not line or line.startswith("*"):
            return

        parts = line.strip().split()
        if not parts:
            return

        message_type = parts[0].rstrip(":")
        data = [self._parse_value(p) for p in parts[1:]]

        instance = None
        try:
            if message_type == 'euler' and len(data) >= 10:
                instance = SbgEkfEulerData(*data)
            elif message_type == 'quat' and len(data) >= 10:
                instance = SbgEkfQuatData(*data)
            elif message_type == 'nav' and len(data) >= 14:
                instance = SbgEkfNavData(*data)
            elif message_type == 'airData' and len(data) >= 6:
                instance = SbgAirData(*data)
            elif message_type == 'imuData' and len(data) >= 8:
                instance = SbgImuData(*data)
            elif message_type == 'gnss1Vel' and len(data) >= 10:
                instance = SbgGnssVelData(*data)
            elif message_type == 'gnss1Pos' and len(data) >= 14:
                instance = SbgGnssPosData(*data)
            elif message_type == 'status' and len(data) >= 5:
                instance = SbgStatusData(*data)
            elif message_type == 'utcTime' and len(data) >= 12:
                instance = SbgUtcTimeData(*data)
            elif message_type == 'gnss1Hdt' and len(data) >= 9:
                instance = SbgGnssHdtData(*data)
            elif message_type == 'gnss1Sat' and len(data) >= 1:
                instance = SbgGnssSatData(*data)
            elif message_type == 'imuShort' and len(data) >= 8:
                instance = SbgImuShortData(*data)
            elif message_type == 'imuFast' and len(data) >= 8:
                instance = SbgImuFastData(*data)
            elif message_type == 'mag' and len(data) >= 5:
                instance = SbgMagData(*data)
            elif message_type == 'diag' and len(data) >= 4:
                instance = SbgDiagData(*data)
            elif message_type == 'velBody' and len(data) >= 7:
                instance = SbgVelBodyData(*data)
        except (TypeError, IndexError) as e:
            # This can happen if the number of data fields is wrong or types are incorrect
            # print(f"Warning: Could not parse data for {message_type} due to a {type(e).__name__}: {e}. Line: '{line.strip()}'")
            return

        if instance:
            self._latest_measurements[message_type] = instance
            if message_type in self._subscriptions:
                for callback in self._subscriptions[message_type]:
                    try:
                        callback(instance)
                    except Exception as e:
                        print(f"Error in callback for {message_type}: {e}")
