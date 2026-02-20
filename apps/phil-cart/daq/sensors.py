from .models.sensor import Sensor

sensors: list[Sensor] = [
    Sensor(
        name="PT A-10 + LJTick-CurrentShunt",
        positive_channel=0,  # AIN0
        negative_channel=1,  # AIN1
        range=10,
        scale=1.0,
        offset=0.0,
    )
]
