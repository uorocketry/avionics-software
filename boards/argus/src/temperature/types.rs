use core::str::FromStr;

use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use heapless::String;
use serde::Serialize;

use crate::adc::driver::types::AnalogChannel;
use crate::sd::csv::types::SerializeCSV;

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
	fn get_header() -> String<255> {
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

/// Defines the association between a thermocouple and the analog input pair it uses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Format, Serialize)]
pub enum ThermocoupleChannel {
	Channel1 = 0,
	Channel2 = 1,
	Channel3 = 2,
	Channel4 = 3,
}

impl ThermocoupleChannel {
	pub fn len() -> usize {
		4
	}

	pub fn from(value: usize) -> Self {
		match value {
			0 => ThermocoupleChannel::Channel1,
			1 => ThermocoupleChannel::Channel2,
			2 => ThermocoupleChannel::Channel3,
			3 => ThermocoupleChannel::Channel4,
			_ => panic!("Invalid thermocouple channel index: {}", value),
		}
	}

	pub fn to_analog_input_channel_pair(&self) -> (AnalogChannel, AnalogChannel) {
		match self {
			ThermocoupleChannel::Channel1 => (AnalogChannel::AIN0, AnalogChannel::AIN1),
			ThermocoupleChannel::Channel2 => (AnalogChannel::AIN2, AnalogChannel::AIN3),
			ThermocoupleChannel::Channel3 => (AnalogChannel::AIN4, AnalogChannel::AIN5),
			ThermocoupleChannel::Channel4 => (AnalogChannel::AIN6, AnalogChannel::AIN7),
		}
	}
}

pub type AdcIndex = usize;
pub type ThermocoupleReadingChannel = Channel<CriticalSectionRawMutex, (AdcIndex, ThermocoupleChannel, ThermocoupleReading), 20>;
