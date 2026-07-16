from typing import Tuple
import numpy as np
import math

_WGS84_A = 6378137.0
_WGS84_F = 1.0 / 298.257223563
_WGS84_E2 = _WGS84_F * (2.0 - _WGS84_F)

def _geodetic_to_ecef(lat_deg: float, lon_deg: float, h_m: float) -> Tuple[float, float, float]:
    lat = math.radians(lat_deg)
    lon = math.radians(lon_deg)
    sin_lat = math.sin(lat)
    cos_lat = math.cos(lat)
    sin_lon = math.sin(lon)
    cos_lon = math.cos(lon)
    N = _WGS84_A / math.sqrt(1.0 - _WGS84_E2 * sin_lat * sin_lat)
    x = (N + h_m) * cos_lat * cos_lon
    y = (N + h_m) * cos_lat * sin_lon
    z = (N * (1.0 - _WGS84_E2) + h_m) * sin_lat
    return x, y, z

def _ecef_to_enu(dx: Tuple[float, float, float], lat0_deg: float, lon0_deg: float) -> Tuple[float, float, float]:
    lat0 = math.radians(lat0_deg)
    lon0 = math.radians(lon0_deg)
    sin_lat = math.sin(lat0)
    cos_lat = math.cos(lat0)
    sin_lon = math.sin(lon0)
    cos_lon = math.cos(lon0)
    dx_x, dx_y, dx_z = dx
    e = -sin_lon * dx_x + cos_lon * dx_y
    n = -sin_lat * cos_lon * dx_x - sin_lat * sin_lon * dx_y + cos_lat * dx_z
    u = cos_lat * cos_lon * dx_x + cos_lat * sin_lon * dx_y + sin_lat * dx_z
    return e, n, u


def compute_az_el(tgt_lat: float, tgt_lon: float, tgt_alt: float) -> Tuple[float, float]:
    x_o, y_o, z_o = _geodetic_to_ecef(
        antenna_lat_deg, antenna_lon_deg, antenna_alt_m
    )
    x_t, y_t, z_t = _geodetic_to_ecef(tgt_lat, tgt_lon, tgt_alt)
    e, n, u = _ecef_to_enu((x_t - x_o, y_t - y_o, z_t - z_o), antenna_lat_deg, antenna_lon_deg)
    az = math.degrees(math.atan2(e, n))
    if az < 0:
        az += 360.0
    el = math.degrees(math.atan2(u, math.hypot(e, n)))
    return az, el

def _normalize_0_360(deg: float) -> float:
    return (deg % 360.0 + 360.0) % 360.0

def _rotation_from_yaw_zero_basic(bearing_deg: float) -> float:
    """Yaw relative to yaw-zero (single-turn), honoring config.yaw_normalization (0to360 or pm180)."""
    raw = bearing_deg - antenna_yaw_zero_deg
    return _normalize_0_360(raw)

def yaw_pitch_from_target(tgt_lat: float, tgt_lon: float, tgt_alt: float) -> Tuple[float, float]:
    print("Calculating Yaw pitch from target")
    az, el = compute_az_el(tgt_lat, tgt_lon, tgt_alt)  # az from North
    yaw_rel = _rotation_from_yaw_zero_basic(az)
    return yaw_rel, el



antenna_yaw_zero_deg = 195

antenna_lat_deg = 47.990527
antenna_lon_deg = -81.851502
antenna_alt_m = 373.0

target_lat_deg =  47.991
target_lon_deg  = -81.851502
target_alt_m = 1000.0


x_o, y_o, z_o = _geodetic_to_ecef(
    antenna_lat_deg, antenna_lon_deg, antenna_alt_m
)
x_t, y_t, z_t = _geodetic_to_ecef(target_lat_deg, target_lon_deg, target_alt_m)
e, n, u = _ecef_to_enu((x_t - x_o, y_t - y_o, z_t - z_o), antenna_lat_deg, antenna_lon_deg)

yaw, pitch = yaw_pitch_from_target(target_lat_deg, target_lon_deg, target_alt_m)
print("Yaw:", yaw)
print("Pitch:", pitch)