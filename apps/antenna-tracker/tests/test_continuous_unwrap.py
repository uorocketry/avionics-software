import pytest

from predictor_service import PredictorConfig, PredictorService


def normalize_0_360(deg: float) -> float:
    return (deg % 360.0 + 360.0) % 360.0


def normalize_pm180(deg: float) -> float:
    d = normalize_0_360(deg)
    return d - 360.0 if d > 180.0 else d


def step_ok(a, b, tol=1e-9):
    return abs(a - b) <= tol


def unwrap_seq_check(bearing_seq, yaw_zero=194.0, seed_mode="pm180"):
    pcfg = PredictorConfig(
        antenna_yaw_zero_deg=yaw_zero,
        yaw_normalization="continuous",
        use_real_gps_ingest=False,
    )
    svc = PredictorService(pcfg)
    svc.reset_yaw_continuity()

    def rot_basic(b):
        raw = b - yaw_zero
        return normalize_pm180(raw) if seed_mode != "0to360" else normalize_0_360(raw)

    def rot_cont(b, ref):
        desired_mod = normalize_0_360(b - yaw_zero)
        ref_mod = normalize_0_360(ref)
        delta = normalize_pm180(desired_mod - ref_mod)
        return ref + delta

    cont_vals = []
    for i, b in enumerate(bearing_seq):
        if i == 0:
            y = rot_basic(b)
        else:
            y = rot_cont(b, cont_vals[-1])
        cont_vals.append(y)

    for i in range(1, len(bearing_seq)):
        bearing_step = bearing_seq[i] - bearing_seq[i - 1]
        expected = normalize_pm180(bearing_step)
        got = cont_vals[i] - cont_vals[i - 1]
        assert step_ok(got, expected, tol=1e-7), f"step {i} expected {expected}, got {got}"

    return cont_vals


@pytest.mark.parametrize(
    "seq",
    [
        list(range(350, 371, 5)) + [0, 5, 10, 15, 20, 25, 30],
        [30, 25, 20, 15, 10, 5, 0, 355, 350, 345, 340],
        list(range(0, 1081, 15)),
        [350, 355, 0, 5, 10, 5, 0, 355, 350, 355, 0, 5],
    ],
)
def test_continuous_unwrap(seq):
    _ = unwrap_seq_check(seq)


def test_clamping_interplay_with_accumulator():
    seq = list(range(0, 721, 10))
    vals = unwrap_seq_check(seq)
    assert vals[-1] > 350.0
