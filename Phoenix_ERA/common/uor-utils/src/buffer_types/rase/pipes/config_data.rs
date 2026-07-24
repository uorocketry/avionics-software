use embassy_time::Instant;

use crate::units::{Distance, Pressure};

#[derive(Clone)]

pub struct ConfigData {
	pub p0_pressure: Pressure,
	pub main_altitude: Distance,
	pub drogue_altitude: Distance,
	pub timestamp: Instant,
}

impl Default for ConfigData {
	fn default() -> Self {
		Self {
			p0_pressure: Pressure::new(1024),
			main_altitude: Distance::new(1500.0),
			drogue_altitude: Distance::new(30000.0),

			timestamp: Instant::now(),
		}
	}
}

impl ConfigData {
	pub fn new(
		p0_pressure: Pressure,
		main_altitude: Distance,
		drogue_altitude: Distance,
	) -> Self {
		Self {
			p0_pressure,
			main_altitude,
			drogue_altitude,
			timestamp: Instant::now(),
		}
	}

	pub fn update(
		&mut self,
		p0_pressure: Pressure,
		main_altitude: Distance,
		drogue_altitude: Distance,
	) {
		self.p0_pressure = p0_pressure;
		self.main_altitude = main_altitude;
		self.drogue_altitude = drogue_altitude;

		self.timestamp = Instant::now();
	}
}
