from datetime import datetime
from peewee import (
    Model,
    ForeignKeyField,
    TimestampField,
    DoubleField,
)
from .sensor import Sensor
from utils.database import database


class DataPoint(Model):
    sensor: Sensor = ForeignKeyField(Sensor, null=True)
    value: float = DoubleField(null=True)
    recorded_at: datetime = TimestampField(default=datetime.now)

    class Meta:
        database = database
        table_name = "datapoints"
        schema = "phil_cart"
