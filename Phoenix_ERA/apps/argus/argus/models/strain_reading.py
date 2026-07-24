from datetime import datetime
from peewee import (
    Model,
    CharField,
    TimestampField,
    DoubleField,
    ForeignKeyField,
    IntegerField,
)
from models.recording_session import HostRecordingSession
from utils.database import database


class StrainReading(Model):
    # The recording session this reading belongs to
    host_session = ForeignKeyField(HostRecordingSession, null=True)

    # Local recording session identifier from the device that took the reading
    local_session = IntegerField(null=True)

    # ADC device index from which the reading was taken
    adc_device = CharField(max_length=255, null=True)

    # Pressure channel from which the reading was taken
    strain_channel = CharField(max_length=255, null=True)

    # Milliseconds since the board's epoch when the reading was recorded
    recorded_at = TimestampField(null=True)

    # Full timestamp of when the reading was stored
    stored_at = TimestampField(default=datetime.now)

    # Wheatstone bridge voltage difference measured in millivolts
    voltage = DoubleField(null=True)

    # Strain reading in microstrain
    strain = DoubleField(null=True)

    class Meta:
        database = database
        table_name = "strain_readings"
