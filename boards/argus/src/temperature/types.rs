use core::str::FromStr;

use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::String;
use serde::Serialize;

use argus::config::AdcDevice;
use argus::sd::csv::types::SerializeCSV;
use argus::sd::types::Line;
use argus::temperature::config::{ThermocoupleChannel, QUEUE_SIZE};

// Represents a linear transformation applied to raw thermocouple voltage readings to get degrees Celsius
// temperature_in_celsius = raw_voltage * gain + offset
#[derive(Debug, Clone, Copy, Format, Serialize, Default)]
pub struct ValueTransformation {
	pub gain: f32,
	pub offset: f32,
}

// Represents a single temperature reading from a thermocouple channel
#[derive(Debug, Clone, Copy, Format, Serialize)]
pub struct ThermocoupleReading {
	// Timestamp of the reading in milliseconds since epoch
	pub timestamp_in_milliseconds: u64,

	// Thermocouple voltage difference measured in millivolts
	pub voltage_in_millivolts: f32,

	// Cold-junction-compensated temperature of the thermocouple in degrees Celsius
	pub compensated_temperature_in_celsius: Option<f32>,

	// Uncompensated temperature of the thermocouple in degrees Celsius
	pub uncompensated_temperature_in_celsius: Option<f32>,

	// Temperature of the cold junction in degrees Celsius
	pub cold_junction_temperature_in_celsius: Option<f32>,
}

impl SerializeCSV for ThermocoupleReading {
	fn get_header() -> Line {
		String::from_str(
			"Timestamp (ms),\
			Voltage (mV),\
			Compensated Temperature (C),\
			Uncompensated Temperature (C),\
			Cold Junction Temperature (C)\n",
		)
		.unwrap()
	}
}

pub type ThermocoupleReadingChannel = Channel<CriticalSectionRawMutex, (AdcDevice, ThermocoupleChannel, ThermocoupleReading), QUEUE_SIZE>;
