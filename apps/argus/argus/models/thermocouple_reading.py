from peewee import Model, CharField, TimestampField, DoubleField, ForeignKeyField
from models.recording_session import RecordingSession
from utils.database import database

from argus.temperature.thermocouple_reading_pb2 import (
    ThermocoupleReading as ThermocoupleReadingProto,
)


class ThermocoupleReading(Model):
    # The recording session this reading belongs to
    session = ForeignKeyField(RecordingSession, null=True)

    # ADC device index from which the reading was taken
    adc_device = CharField(max_length=255, null=True)

    # Thermocouple channel from which the reading was taken
    thermocouple_channel = CharField(max_length=255, null=True)

    # Timestamp of the reading in milliseconds since epoch
    timestamp = TimestampField(null=True)

    # Thermocouple voltage difference measured in millivolts
    voltage = DoubleField(null=True)

    # Cold-junction-compensated temperature of the thermocouple in degrees Celsius
    compensated_temperature = DoubleField(null=True)

    # Uncompensated temperature of the thermocouple in degrees Celsius
    uncompensated_temperature = DoubleField(null=True)

    # Temperature of the cold junction in degrees Celsius
    cold_junction_temperature = DoubleField(null=True)

    class Meta:
        database = database  # This should be your Peewee database instance
        table_name = "thermocouple_readings"

    @staticmethod
    def from_protobuf(proto: ThermocoupleReadingProto):
        return ThermocoupleReading(
            adc_device=proto.adc_device,
            thermocouple_channel=proto.thermocouple_channel,
            timestamp=int(proto.timestamp),
            voltage=proto.voltage,
            compensated_temperature=proto.compensated_temperature,
            uncompensated_temperature=proto.uncompensated_temperature,
            cold_junction_temperature=proto.cold_junction_temperature,
        )


database.create_tables([ThermocoupleReading])
