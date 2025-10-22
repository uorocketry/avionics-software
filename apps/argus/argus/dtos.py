from models.thermocouple_reading import ThermocoupleReading
from argus.temperature.thermocouple_reading_pb2 import (
    ThermocoupleReading as ThermocoupleReadingProto,
)

message_type_to_model = {
    ThermocoupleReadingProto: ThermocoupleReading,
}
