# Temperature
from models.thermocouple_reading import ThermocoupleReading
from messages.argus.temperature.thermocouple_reading_pb2 import (
    ThermocoupleReading as ThermocoupleReadingProto,
)

# Pressure
from models.pressure_reading import PressureReading
from messages.argus.pressure.pressure_reading_pb2 import (
    PressureReading as PressureReadingProto,
)

# Strain
from models.strain_reading import StrainReading
from messages.argus.strain.strain_reading_pb2 import (
    StrainReading as StrainReadingProto,
)

proto_to_model = {
    ThermocoupleReadingProto: lambda proto: ThermocoupleReading(
        local_session=proto.local_session,
        adc_device=proto.adc_device,
        thermocouple_channel=proto.thermocouple_channel,
        recorded_at=int(proto.recorded_at),
        voltage=proto.voltage,
        compensated_temperature=proto.compensated_temperature,
        uncompensated_temperature=proto.uncompensated_temperature,
        cold_junction_temperature=proto.cold_junction_temperature,
    ),
    PressureReadingProto: lambda proto: PressureReading(
        local_session=proto.local_session,
        adc_device=proto.adc_device,
        pressure_channel=proto.pressure_channel,
        recorded_at=int(proto.recorded_at),
        voltage=proto.voltage,
        pressure=proto.pressure,
        temperature=proto.temperature,
    ),
    StrainReadingProto: lambda proto: StrainReading(
        local_session=proto.local_session,
        adc_device=proto.adc_device,
        strain_channel=proto.strain_channel,
        recorded_at=int(proto.recorded_at),
        voltage=proto.voltage,
        strain=proto.strain,
    ),
}
