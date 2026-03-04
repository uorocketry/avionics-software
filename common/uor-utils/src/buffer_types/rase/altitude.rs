use embassy_time::Instant;

use crate::units::Distance;

#[derive(Clone)]

pub struct AltitudeData {
	pub altitude: Distance,
	pub timestamp: Instant,
}

impl Default for AltitudeData {
	fn default() -> Self {
		Self {
			altitude: Default::default(),
			timestamp: Instant::now(),
		}
	}
}

impl AltitudeData {
	pub fn new(altitude: Distance) -> Self {
		Self {
			altitude: altitude,
			timestamp: Instant::now(),
		}
	}

	pub fn update(
		&mut self,
		altitude: Distance,
	) {
		self.altitude = altitude;
		self.timestamp = Instant::now();
	}
}
