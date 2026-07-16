import math
import pytest

from predictor_service import PredictorConfig, PredictorService
from antenna_service import AntennaConfig, AntennaService


def normalize_0_360(deg: float) -> float:
    return (deg % 360.0 + 360.0) % 360.0


def normalize_pm180(deg: float) -> float:
    d = normalize_0_360(deg)
    return d - 360.0 if d > 180.0 else d


def destination_point(lat_deg, lon_deg, bearing_deg, distance_m, radius_m=6371000.0):
    lat1 = math.radians(lat_deg)
    lon1 = math.radians(lon_deg)
    brng = math.radians(bearing_deg)
    dr = distance_m / radius_m
    sin_lat1 = math.sin(lat1)
    cos_lat1 = math.cos(lat1)
    sin_dr = math.sin(dr)
    cos_dr = math.cos(dr)
    sin_lat2 = sin_lat1 * cos_dr + cos_lat1 * sin_dr * math.cos(brng)
    lat2 = math.asin(sin_lat2)
    y = math.sin(brng) * sin_dr * cos_lat1
    x = cos_dr - sin_lat1 * sin_lat2
    lon2 = lon1 + math.atan2(y, x)
    return math.degrees(lat2), (math.degrees(lon2) + 540) % 360 - 180


def expected_el(distance_m: float, alt_delta_m: float) -> float:
    return math.degrees(math.atan2(alt_delta_m, max(distance_m, 1e-9)))


bearings = [0, 45, 90, 135, 180, 225, 270, 315]
distances = [1.0, 100.0, 1000.0]
alt_deltas = [0.0, +5.0, -5.0, +100.0, -100.0]


@pytest.mark.parametrize("d", distances)
@pytest.mark.parametrize("dh", alt_deltas)
@pytest.mark.parametrize("b", bearings)
def test_az_el_yaw_and_commands(b, d, dh):
    pcfg = PredictorConfig(yaw_normalization="pm180", use_real_gps_ingest=False)
    svc = PredictorService(pcfg)
    acfg = AntennaConfig(use_real_hardware=False)

    tgt_lat, tgt_lon = destination_point(pcfg.antenna_lat_deg, pcfg.antenna_lon_deg, b, d)
    tgt_alt = pcfg.antenna_alt_m + dh

    az, el = svc.compute_az_el(tgt_lat, tgt_lon, tgt_alt)

    az_exp = b % 360.0
    el_exp = expected_el(d, dh)

    tol_az = 0.5
    tol_el = 0.5
    assert abs((az - az_exp + 540) % 360 - 180) <= tol_az
    assert abs(el - el_exp) <= tol_el

    yaw = normalize_pm180(az - pcfg.antenna_yaw_zero_deg)
    yaw_exp = normalize_pm180(az_exp - pcfg.antenna_yaw_zero_deg)
    assert abs(yaw - yaw_exp) <= tol_az

    yaw_lo, yaw_hi = acfg.limits_yaw_deg
    pit_lo, pit_hi = acfg.limits_pitch_deg
    yaw_cmd = max(yaw_lo, min(yaw_hi, yaw))
    pit_cmd = max(pit_lo, min(pit_hi, el))

    yaw_turns = (acfg.dir_yaw * yaw_cmd / 360.0) * acfg.gear_ratio_yaw
    pit_turns = (acfg.dir_pitch * pit_cmd / 360.0) * acfg.gear_ratio_pitch

    ant = AntennaService(acfg)
    ok = ant.set_position(pitch=el, yaw=yaw)
    assert ok
    pos = ant.get_position()
    assert math.isclose(pos["yaw"], yaw_cmd, abs_tol=1e-6)
    assert math.isclose(pos["pitch"], pit_cmd, abs_tol=1e-6)
