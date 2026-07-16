#!/usr/bin/env python3
"""
Example Python script that demonstrates how gps-ingest can be used in a pipeline.
This simulates the "predictor" component from the antenna tracker architecture.
"""

import json
import subprocess
import sys
import time
import shlex
import select
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime, timezone
import math
import numpy as np

# Antenna tracker location (observer)
ANTENNA_LAT_DEG = 48.48058344683317
ANTENNA_LON_DEG = -81.33124126482966
ANTENNA_ALT_M = 300.0

# Toggle between real gps-ingest and test mode (easily tweakable)
USE_REAL_GPS_INGEST = False
# Test coordinates for the target (rocket) when not using real gps-ingest
TEST_TARGET_LAT_DEG = 48.48158344683317
TEST_TARGET_LON_DEG = -81.33124126482966
TEST_TARGET_ALT_M = 301.0

# Antenna yaw-zero reference and normalization (easily tweakable)
# Yaw-zero is the mechanical zero direction; with South as zero we set 180° from North
ANTENNA_YAW_ZERO_DEG = 180.0  # due South
ROTATION_NORMALIZATION = "pm180"  # "pm180" for [-180,180], or "0to360" for [0,360)

# History configuration
HISTORY_MAX_SAMPLES = 200
STREAM_TIMEOUT_S = 15.0
TARGET_STREAM_SAMPLES = 10  # try to collect up to this many per run if available

# WGS84 ellipsoid constants
_WGS84_A = 6378137.0  # semi-major axis [m]
_WGS84_F = 1.0 / 298.257223563  # flattening
_WGS84_E2 = _WGS84_F * (2.0 - _WGS84_F)  # eccentricity squared

@dataclass
class GpsPoint:
    latitude: float
    longitude: float
    altitude: float
    timestamp: str
    source: str

@dataclass
class Prediction:
    target_lat: float
    target_lon: float
    target_alt: float
    confidence: float
    prediction_time: str

class GpsPredictor:
    """Simple GPS predictor that demonstrates pipeline usage"""

    def __init__(self, gps_ingest_path: str = "cargo run -p gps-ingest"):
        self.gps_ingest_path = gps_ingest_path
        self.last_position: Optional[GpsPoint] = None
        # Each entry: (t_epoch_s, lat, lon, alt)
        self.history: List[tuple[float, float, float, float]] = []

    def _add_sample(self, pos: GpsPoint) -> None:
        t = time.time()
        self.history.append((t, pos.latitude, pos.longitude, pos.altitude))
        if len(self.history) > HISTORY_MAX_SAMPLES:
            self.history = self.history[-HISTORY_MAX_SAMPLES:]
        self.last_position = pos

    def _add_sample_from_dict(self, d: Dict) -> None:
        pos = GpsPoint(
            latitude=d['lat'],
            longitude=d['lon'],
            altitude=d['altitude_m'],
            timestamp=d['ts'],
            source=d.get('source', 'gps-ingest')
        )
        self._add_sample(pos)

    def get_latest_gps_data(self) -> Optional[Dict]:
        """Get GPS data by streaming stdout; accumulate multiple samples for regression when possible.
        If USE_REAL_GPS_INGEST is False, return test coordinates and add them to history.
        """
        try:
            if not USE_REAL_GPS_INGEST:
                # Test mode: return synthetic sample and add to history
                now = datetime.now(timezone.utc).isoformat()
                print(
                    f"[TEST MODE] Using fixed coordinates lat={TEST_TARGET_LAT_DEG}, lon={TEST_TARGET_LON_DEG}, alt={TEST_TARGET_ALT_M}m"
                )
                d = {
                    "lat": TEST_TARGET_LAT_DEG,
                    "lon": TEST_TARGET_LON_DEG,
                    "altitude_m": TEST_TARGET_ALT_M,
                    "ts": now,
                    "source": "test"
                }
                self._add_sample_from_dict(d)
                return d

            cmd = shlex.split(self.gps_ingest_path)
            proc = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,  # line-buffered
                cwd=".."
            )

            start = time.monotonic()
            timeout_s = STREAM_TIMEOUT_S
            samples_collected = 0
            last_valid: Optional[Dict] = None

            assert proc.stdout is not None
            while True:
                # Break if timed out
                if time.monotonic() - start > timeout_s:
                    break

                rlist, _, _ = select.select([proc.stdout], [], [], 0.5)
                if not rlist:
                    # Also break if process has exited and there's nothing to read
                    if proc.poll() is not None:
                        break
                    continue

                line = proc.stdout.readline()
                if not line:
                    # EOF
                    if proc.poll() is not None:
                        break
                    continue

                line = line.strip()
                if not line:
                    continue

                # Expect JSON lines from gps-ingest; skip logs
                if not (line.startswith("{") and line.endswith("}")):
                    continue

                try:
                    candidate = json.loads(line)
                except json.JSONDecodeError:
                    continue

                if (
                    candidate.get("lat") is not None
                    and candidate.get("lon") is not None
                    and candidate.get("altitude_m") is not None
                    and candidate.get("ts") is not None
                ):
                    # Accept both gps-ingest and test sources for accumulation
                    self._add_sample_from_dict(candidate)
                    last_valid = candidate
                    samples_collected += 1
                    if samples_collected >= TARGET_STREAM_SAMPLES:
                        break

            # Ensure process is stopped
            if proc.poll() is None:
                try:
                    proc.terminate()
                    try:
                        proc.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                except Exception:
                    pass

            return last_valid

        except FileNotFoundError as e:
            print(f"gps-ingest not found: {e}", file=sys.stderr)
            return None
        except Exception as e:
            print(f"Error getting GPS data: {e}", file=sys.stderr)
            return None

    def predict_next_position(self, current_pos: GpsPoint) -> Prediction:
        """Predict next position (+1s) using linear regression over accumulated history."""
        # Ensure the current point is included
        self._add_sample(current_pos)

        if len(self.history) < 2:
            # Not enough data; return current as predicted
            t_iso = datetime.now(timezone.utc).isoformat()
            return Prediction(
                target_lat=current_pos.latitude,
                target_lon=current_pos.longitude,
                target_alt=current_pos.altitude,
                confidence=0.5,
                prediction_time=t_iso,
            )

        # Prepare data for regression
        times = np.array([t for (t, _, _, _) in self.history], dtype=float)
        lats = np.array([lat for (_, lat, _, _) in self.history], dtype=float)
        lons = np.array([lon for (_, _, lon, _) in self.history], dtype=float)
        alts = np.array([alt for (_, _, _, alt) in self.history], dtype=float)

        # Normalize time to improve conditioning
        t0 = times[0]
        t_rel = times - t0
        t_future = (times[-1] + 1.0) - t0  # predict 1 second ahead

        try:
            lat_slope, lat_intercept = np.polyfit(t_rel, lats, 1)
            lon_slope, lon_intercept = np.polyfit(t_rel, lons, 1)
            alt_slope, alt_intercept = np.polyfit(t_rel, alts, 1)

            pred_lat = lat_slope * t_future + lat_intercept
            pred_lon = lon_slope * t_future + lon_intercept
            pred_alt = alt_slope * t_future + alt_intercept
            confidence = 0.9 if len(self.history) >= 5 else 0.7
        except Exception:
            # Fallback if polyfit fails (e.g., numerical issues)
            pred_lat = current_pos.latitude
            pred_lon = current_pos.longitude
            pred_alt = current_pos.altitude
            confidence = 0.5

        return Prediction(
            target_lat=pred_lat,
            target_lon=pred_lon,
            target_alt=pred_alt,
            confidence=confidence,
            prediction_time=datetime.now(timezone.utc).isoformat()
        )

    @staticmethod
    def _geodetic_to_ecef(lat_deg: float, lon_deg: float, h_m: float) -> tuple[float, float, float]:
        lat = math.radians(lat_deg)
        lon = math.radians(lon_deg)
        sin_lat = math.sin(lat)
        cos_lat = math.cos(lat)
        sin_lon = math.sin(lon)
        cos_lon = math.cos(lon)
        # Prime vertical radius of curvature
        N = _WGS84_A / math.sqrt(1.0 - _WGS84_E2 * sin_lat * sin_lat)
        x = (N + h_m) * cos_lat * cos_lon
        y = (N + h_m) * cos_lat * sin_lon
        z = (N * (1.0 - _WGS84_E2) + h_m) * sin_lat
        return x, y, z

    @staticmethod
    def _ecef_to_enu(dx: tuple[float, float, float], lat0_deg: float, lon0_deg: float) -> tuple[float, float, float]:
        lat0 = math.radians(lat0_deg)
        lon0 = math.radians(lon0_deg)
        sin_lat = math.sin(lat0)
        cos_lat = math.cos(lat0)
        sin_lon = math.sin(lon0)
        cos_lon = math.cos(lon0)
        dx_x, dx_y, dx_z = dx
        # Rotation from ECEF delta to ENU at observer
        e = -sin_lon * dx_x + cos_lon * dx_y
        n = -sin_lat * cos_lon * dx_x - sin_lat * sin_lon * dx_y + cos_lat * dx_z
        u =  cos_lat * cos_lon * dx_x + cos_lat * sin_lon * dx_y + sin_lat * dx_z
        return e, n, u

    @staticmethod
    def normalize_angle_0_360(deg: float) -> float:
        return (deg % 360.0 + 360.0) % 360.0

    @staticmethod
    def normalize_angle_pm180(deg: float) -> float:
        d = GpsPredictor.normalize_angle_0_360(deg)
        return d - 360.0 if d > 180.0 else d

    @staticmethod
    def initial_bearing_deg(lat1_deg: float, lon1_deg: float, lat2_deg: float, lon2_deg: float) -> float:
        """Great-circle initial bearing from point 1 to point 2, degrees cw from North."""
        lat1 = math.radians(lat1_deg)
        lat2 = math.radians(lat2_deg)
        dlon = math.radians(lon2_deg - lon1_deg)
        x = math.sin(dlon) * math.cos(lat2)
        y = math.cos(lat1) * math.sin(lat2) - math.sin(lat1) * math.cos(lat2) * math.cos(dlon)
        brng = math.degrees(math.atan2(x, y))
        return GpsPredictor.normalize_angle_0_360(brng)

    def compute_az_el(self, obs_lat: float, obs_lon: float, obs_alt: float,
                       tgt_lat: float, tgt_lon: float, tgt_alt: float) -> tuple[float, float]:
        """Compute azimuth (yaw, degrees cw from North) and elevation (pitch, degrees) from observer to target."""
        x_o, y_o, z_o = self._geodetic_to_ecef(obs_lat, obs_lon, obs_alt)
        x_t, y_t, z_t = self._geodetic_to_ecef(tgt_lat, tgt_lon, tgt_alt)
        dx = (x_t - x_o, y_t - y_o, z_t - z_o)
        e, n, u = self._ecef_to_enu(dx, obs_lat, obs_lon)
        az_rad = math.atan2(e, n)  # 0 at North, +East
        el_rad = math.atan2(u, math.hypot(e, n))
        az_deg = math.degrees(az_rad)
        if az_deg < 0:
            az_deg += 360.0
        el_deg = math.degrees(el_rad)
        return az_deg, el_deg

    def rotation_from_yaw_zero(self, bearing_deg: float, yaw_zero_deg: float, mode: str = "pm180") -> float:
        raw = bearing_deg - yaw_zero_deg
        if mode == "0to360":
            return GpsPredictor.normalize_angle_0_360(raw)
        return GpsPredictor.normalize_angle_pm180(raw)

    def run_prediction_cycle(self):
        """Main prediction cycle - simulates continuous operation"""
        print("=== GPS Predictor Starting ===")
        print("Getting latest GPS data...")

        data = self.get_latest_gps_data()

        if not data:
            print("No valid GPS data available")
            return

        # Create GPS point from the new/last data
        current_pos = GpsPoint(
            latitude=data['lat'],
            longitude=data['lon'],
            altitude=data['altitude_m'],
            timestamp=data['ts'],
            source=data.get('source', 'gps-ingest')
        )

        print(f"Current Position: {current_pos.latitude:.6f}, {current_pos.longitude:.6f}")
        print(f"Altitude: {current_pos.altitude:.1f}m")
        print(f"Source: {current_pos.source}")
        print(f"Timestamp: {current_pos.timestamp}")

        # Compute az/el for current position
        az_deg, el_deg = self.compute_az_el(
            ANTENNA_LAT_DEG, ANTENNA_LON_DEG, ANTENNA_ALT_M,
            current_pos.latitude, current_pos.longitude, current_pos.altitude
        )
        # Compute great-circle initial bearing (from North, cw) and rotation from yaw-zero=South
        bearing_deg = self.initial_bearing_deg(
            ANTENNA_LAT_DEG, ANTENNA_LON_DEG,
            current_pos.latitude, current_pos.longitude
        )
        rotation_deg = self.rotation_from_yaw_zero(
            bearing_deg, ANTENNA_YAW_ZERO_DEG, ROTATION_NORMALIZATION
        )
        print(f"\nPointing (current):")
        print(f"  Yaw (azimuth): {az_deg:.2f}°  |  Pitch (elevation): {el_deg:.2f}°")
        print(f"  Great-circle bearing: {bearing_deg:.2f}° from North")
        print(f"  Antenna rotation (yaw-zero=South): {rotation_deg:.2f}° [{ROTATION_NORMALIZATION}]")

        # Predict next position (+1s) using regression over accumulated history
        prediction = self.predict_next_position(current_pos)

        print(f"\nPrediction (1 second ahead) [linear regression]:")
        print(f"  Target: {prediction.target_lat:.6f}, {prediction.target_lon:.6f}")
        print(f"  Altitude: {prediction.target_alt:.1f}m")
        print(f"  Confidence: {prediction.confidence:.2f}")

        # Compute az/el, bearing and rotation for predicted position
        az_pred_deg, el_pred_deg = self.compute_az_el(
            ANTENNA_LAT_DEG, ANTENNA_LON_DEG, ANTENNA_ALT_M,
            prediction.target_lat, prediction.target_lon, prediction.target_alt
        )
        bearing_pred_deg = self.initial_bearing_deg(
            ANTENNA_LAT_DEG, ANTENNA_LON_DEG,
            prediction.target_lat, prediction.target_lon
        )
        rotation_pred_deg = self.rotation_from_yaw_zero(
            bearing_pred_deg, ANTENNA_YAW_ZERO_DEG, ROTATION_NORMALIZATION
        )
        print(f"\nPointing (predicted +1s):")
        print(f"  Yaw (azimuth): {az_pred_deg:.2f}°  |  Pitch (elevation): {el_pred_deg:.2f}°")
        print(f"  Great-circle bearing: {bearing_pred_deg:.2f}° from North")
        print(f"  Antenna rotation (yaw-zero=South): {rotation_pred_deg:.2f}° [{ROTATION_NORMALIZATION}]")

        # In a real system, this would be sent to the antenna tracker API
        print(f"\nSending to antenna tracker API...")
        print(
            f"  COMMAND: TRACK yaw={az_pred_deg:.2f} pitch={el_pred_deg:.2f} rotation={rotation_pred_deg:.2f} (at +1s)"
        )

        self.last_position = current_pos

def main():
    """Main function demonstrating the pipeline"""
    predictor = GpsPredictor()

    try:
        predictor.run_prediction_cycle()
    except KeyboardInterrupt:
        print("\nPrediction cycle interrupted")
    except Exception as e:
        print(f"Error in prediction cycle: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
