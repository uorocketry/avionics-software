use core::str::FromStr;

use csv::SerializeCSV;
use defmt::Format;
use messages::argus::strain::strain_reading::StrainReading as StrainReadingProtobuf;
use serde::{Deserialize, Serialize};

use crate::adc::types::AdcDevice;
use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::Line;
use crate::strain::types::StrainChannel;

// Represents a single strain reading from a strain channel
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct StrainReading {
	// Local session from the device that took the reading
	pub local_session: Option<i32>,

	// ADC device from which the reading was taken
	pub adc_device: AdcDevice,

	// Identifier for the strain within the ADC device
	pub strain_channel: StrainChannel,

	// Milliseconds since the board's epoch when the reading was recorded
	pub recorded_at: u64,

	// Voltage difference measured at the strain sensor wheatstone bridge in millivolts
	pub voltage: f32,

	// Strain reading
	pub strain: f64,
}

impl SerializeCSV<MAX_LINE_LENGTH> for StrainReading {
	fn get_csv_header() -> Line {
		Line::from_str(
			"Local Session #,\
			ADC Device,\
			Strain Channel,\
			Timestamp (ms),\
			Voltage (mV),\
			Strain",
		)
		.unwrap()
	}
}

impl StrainReading {
	// Convert to the protobuf representation
	pub fn to_protobuf(&self) -> StrainReadingProtobuf {
		StrainReadingProtobuf {
			local_session: self.local_session,
			adc_device: self.adc_device.to_protobuf() as i32,
			strain_channel: self.strain_channel.to_protobuf() as i32,
			recorded_at: self.recorded_at,
			voltage: self.voltage,
			strain: self.strain,
			temperature: self.temperature,
		}
	}
}
