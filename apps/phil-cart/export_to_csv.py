import csv
import logging
import utils.logger
import re
from datetime import datetime
from pathlib import Path

from daq.models.datapoint import DataPoint
from daq.models.sensor import Sensor
from daq.sensors import sensors
from utils.database import database

logger = logging.getLogger(__name__)

if __name__ == "__main__":
    database.connect(reuse_if_open=True)

    # Ensure the output directory exists
    output_dir = Path("logs") / datetime.now().strftime("%Y-%m-%d_%H-%M-%S") / "sensors"
    output_dir.mkdir(parents=True, exist_ok=True)

    try:
        for sensor in sensors:
            file_name = (
                f"{re.sub(r'[^A-Za-z0-9._-]+', '_', sensor.name).strip('._')}.csv"
            )
            output_file = output_dir / file_name

            datapoints = (
                DataPoint.select()
                .where(DataPoint.sensor == sensor)
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

            logger.info('Exported sensor "%s" data to %s.', sensor.name, output_file)
    finally:
        if not database.is_closed():
            database.close()
