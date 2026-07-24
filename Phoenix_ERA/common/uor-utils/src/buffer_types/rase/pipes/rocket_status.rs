use embassy_time::Instant;

use crate::types::rocket_types::RocketState;

#[derive(Clone)]
pub struct RocketStatusData {
	pub rocket_state: RocketState,
	pub timestamp: Instant,
}

impl Default for RocketStatusData {
	fn default() -> Self {
		Self {
			rocket_state: RocketState::InFlight,
			timestamp: Instant::now(),
		}
	}
}

impl RocketStatusData {
	pub fn new(rocket_state: RocketState) -> Self {
		Self {
			rocket_state: rocket_state,
			timestamp: Instant::now(),
		}
	}

	pub fn update(
		&mut self,
		rocket_state: RocketState,
	) {
		self.rocket_state = rocket_state;
		self.timestamp = Instant::now();
	}
}
