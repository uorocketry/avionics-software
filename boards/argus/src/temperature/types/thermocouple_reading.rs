use core::str::FromStr;

use csv::SerializeCSV;
use defmt::Format;
use messages::argus::temperature::thermocouple_reading::ThermocoupleReading as ThermocoupleReadingProtobuf;
use serde::{Deserialize, Serialize};

use crate::adc::types::AdcDevice;
use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::Line;
use crate::temperature::types::ThermocoupleChannel;

// Represents a single temperature reading from a thermocouple channel
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct ThermocoupleReading {
	// ADC device index from which the reading was taken
	pub adc_device: AdcDevice,

	// Thermocouple channel from which the reading was taken
	pub thermocouple_channel: ThermocoupleChannel,

	// Timestamp of the reading in milliseconds since epoch
	pub timestamp: u64,

	// Thermocouple voltage difference measured in millivolts
	pub voltage: f32,

	// Cold-junction-compensated temperature of the thermocouple in degrees Celsius
	pub compensated_temperature: f64,

	// Uncompensated temperature of the thermocouple in degrees Celsius
	pub uncompensated_temperature: f64,

	// Temperature of the cold junction in degrees Celsius
	pub cold_junction_temperature: f32,
}

impl SerializeCSV<MAX_LINE_LENGTH> for ThermocoupleReading {
	fn get_csv_header() -> Line {
		Line::from_str(
			"ADC Device,\
			Thermocouple Channel,\
			Timestamp (ms),\
			Voltage (mV),\
			Compensated Temperature (C),\
			Uncompensated Temperature (C),\
			Cold Junction Temperature (C)\n",
		)
		.unwrap()
	}
}

impl ThermocoupleReading {
	// Convert to the protobuf representation
	pub fn to_protobuf(&self) -> ThermocoupleReadingProtobuf {
		ThermocoupleReadingProtobuf {
			adc_device: self.adc_device.to_protobuf() as i32,
			thermocouple_channel: self.thermocouple_channel.to_protobuf() as i32,
			timestamp: self.timestamp,
			voltage: self.voltage,
			compensated_temperature: self.compensated_temperature,
			uncompensated_temperature: self.uncompensated_temperature,
			cold_junction_temperature: self.cold_junction_temperature,
		}
	}
}
