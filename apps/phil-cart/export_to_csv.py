import logging
import utils.logger
import argparse
import csv
from datetime import datetime, timedelta
from pathlib import Path
import re
import pandas

from daq.models.datapoint import DataPoint
from daq.sensors import sensors
from utils.database import database

logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    default_to = datetime.now()
    default_from = default_to - timedelta(days=1)

    parser = argparse.ArgumentParser(
        description="Export sensor datapoints to CSV within a time range."
    )
    parser.add_argument(
        "--from",
        dest="from_time",
        type=lambda x: pandas.to_datetime(x).to_pydatetime(),
        default=default_from,
        help="Start of time range (inclusive). Default: now - 1 day.",
    )
    parser.add_argument(
        "--to",
        dest="to_time",
        type=lambda x: pandas.to_datetime(x).to_pydatetime(),
        default=default_to,
        help="End of time range (inclusive). Default: now.",
    )
    args = parser.parse_args()

    if args.from_time > args.to_time:
        parser.error("--from must be earlier than or equal to --to.")

    return args


if __name__ == "__main__":
    args = parse_args()
    database.connect(reuse_if_open=True)

    output_dir = Path("logs") / datetime.now().strftime("%Y-%m-%d_%H-%M-%S") / "sensors"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        for sensor in sensors:
            sensor.ensure_exists()
            file_name = (
                f"{re.sub(r'[^A-Za-z0-9._-]+', '_', sensor.name).strip('._')}.csv"
            )
            output_file = output_dir / file_name

            datapoints = (
                DataPoint.select()
                .where(
                    (DataPoint.sensor == sensor)
                    & (DataPoint.recorded_at >= args.from_time)
                    & (DataPoint.recorded_at <= args.to_time)
                )
                .order_by(DataPoint.recorded_at)
            )

            with output_file.open("w", newline="", encoding="utf-8") as csvfile:
                writer = csv.writer(csvfile)
                writer.writerow(["id", "sensor_id", "value", "recorded_at"])

                for datapoint in datapoints:
                    recorded_at = (
                        datapoint.recorded_at.isoformat()
                        if hasattr(datapoint.recorded_at, "isoformat")
                        else str(datapoint.recorded_at)
                    )
                    writer.writerow(
                        [
                            datapoint.id,
                            datapoint.sensor_id,
                            datapoint.value,
                            recorded_at,
                        ]
                    )

            logger.info('Exported sensor "%s" data to %s', sensor.name, output_file)
    finally:
        if not database.is_closed():
            database.close()


def parse_datetime_arg(value: str) -> datetime:
    normalized = value.strip()
    if normalized.endswith("Z"):
        normalized = normalized[:-1] + "+00:00"
    try:
        return datetime.fromisoformat(normalized)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(
            (
                f"Invalid datetime: {value!r}. "
                "Use ISO-8601, e.g. 2026-02-12 or 2026-02-12T14:21:06."
            )
        ) from exc
