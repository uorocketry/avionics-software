use core::str::FromStr;

use defmt::Format;
use serde::{Deserialize, Serialize};
use uor_utils::csv::SerializeCSV;
use uor_utils::messages::argus::pressure::pressure_reading::PressureReading as PressureReadingProtobuf;

use crate::adc::types::AdcDevice;
use crate::pressure::types::PressureChannel;
use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::Line;

// Represents a single pressure reading from a pressure channel
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct PressureReading {
	// Local session from the device that took the reading
	pub local_session: Option<i32>,

	// ADC device from which the reading was taken
	pub adc_device: AdcDevice,

	// Identifier for the pressure within the ADC device
	pub pressure_channel: PressureChannel,

	// Milliseconds since the board's epoch when the reading was recorded
	pub recorded_at: u64,

	// Voltage difference measured at the pressure sensor wheatstone bridge in millivolts
	pub voltage: f32,

	// Pressure reading in psi
	pub pressure: f64,

	// Temperature of the manifold from the NTC resistor at the time of the recording in degrees Celsius
	pub temperature: f64,
}

impl SerializeCSV<MAX_LINE_LENGTH> for PressureReading {
	fn get_csv_header() -> Line {
		Line::from_str(
			"Local Session #,\
			ADC Device,\
			Pressure Channel,\
			Timestamp (ms),\
			Voltage (mV),\
			Pressure (psi),\
			Manifold Temperature (C)",
		)
		.unwrap()
	}
}

impl PressureReading {
	// Convert to the protobuf representation
	pub fn to_protobuf(&self) -> PressureReadingProtobuf {
		PressureReadingProtobuf {
			local_session: self.local_session,
			adc_device: self.adc_device.to_protobuf() as i32,
			pressure_channel: self.pressure_channel.to_protobuf() as i32,
			recorded_at: self.recorded_at,
			voltage: self.voltage,
			pressure: self.pressure,
			temperature: self.temperature,
		}
	}
}
