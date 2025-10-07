use core::str::FromStr;

use defmt::Format;
use serde::{Deserialize, Serialize};

use crate::sd::csv::types::SerializeCSV;
use crate::sd::types::Line;

// Represents a single pressure reading from a pressure channel
#[derive(Debug, Clone, Copy, Format, Serialize, Deserialize)]
pub struct PressureReading {
	// Timestamp of the reading in milliseconds since epoch
	pub timestamp: u64,

	// pressure voltage difference measured in millivolts
	pub voltage: f32,

	// pressure calculated in psi
	pub pressure: f32,

	// manifold temperature at which this pressure reading was taken
	pub temperature: f32,
}

impl SerializeCSV for PressureReading {
	fn get_csv_header() -> Line {
		Line::from_str(
			"Timestamp (ms),\
			Voltage (mV),\
			Pressure (psi),\
			Manifold Temperature (C)\n",
		)
		.unwrap()
	}
}
