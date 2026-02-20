from .service import LabjackIngestorService
from .models.datapoint import DataPoint
from .sensors import sensors


def main():
    labjack_service = LabjackIngestorService(mock=True)
    labjack_service.connect()

    labjack_service.start_transaction()

    # Automatic sampling rate
    labjack_service.write_register("AIN_ALL_SAMPLING_RATE_HZ", 0)

    # Automatic resolution index
    labjack_service.write_register("AIN_ALL_RESOLUTION_INDEX", 0)

    for sensor in sensors:
        sensor.ensure_exists()
        labjack_service.configure_sensor(sensor)

    labjack_service.commit_transaction()

    while True:
        for sensor in sensors:
            value = labjack_service.read_sensor(sensor)
            if value is None:
                continue
            DataPoint(sensor=sensor, value=value).save()
