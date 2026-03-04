use embassy_time::Instant;

use crate::{
	linear_algebra::vectors::{VectorR3Custom, VectorR3F},
	units::{Angle, Distance},
};

#[derive(Clone)]

pub struct PoseData {
	// X, Y, Z,
	pub gyro: VectorR3Custom<Angle>,
	pub acceleration: VectorR3Custom<Distance>,
	pub velocity: VectorR3Custom<Distance>,
	pub timestamp: Instant,
}

impl Default for PoseData {
	fn default() -> Self {
		Self {
			timestamp: Instant::now(),
			gyro: VectorR3Custom::zero_vector(),
			acceleration: VectorR3Custom::zero_vector(),
			velocity: VectorR3Custom::zero_vector(),
		}
	}
}

impl PoseData {
	pub fn new(
		gyro: VectorR3Custom<Angle>,
		acceleration: VectorR3Custom<Distance>,
		velocity: VectorR3Custom<Distance>,
	) -> Self {
		Self {
			gyro,
			acceleration,
			velocity,
			timestamp: Instant::now(),
		}
	}

	pub fn update(&mut self) {
		self.timestamp = Instant::now();
	}
}
