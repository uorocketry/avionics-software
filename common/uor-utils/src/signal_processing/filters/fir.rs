// Contains all Finite Impulse Response (FIR) filters https://en.wikipedia.org/wiki/Finite_impulse_response

use crate::utils::data_structures::ring_buffer::RingBuffer;

pub struct MovingAverageFilter<const N: usize> {
	internal: RingBuffer<f32, N>,
}

impl<const N: usize> MovingAverageFilter<N> {
	pub fn new() -> Self {
		MovingAverageFilter { internal: RingBuffer::new() }
	}

	pub fn push(
		&mut self,
		value: f32,
	) {
		self.internal.push(value);
	}

	// TODO: This is currently bugged, the ring buffer initializes with all 0s, so the average will be extremely low until all positions are occupied
	pub fn get_average(&mut self) -> f32 {
		let mut average = 0.0;
		for i in self.internal.internal_buffer.clone() {
			average += i;
		}
		average / (self.internal.internal_buffer.len() as f32)
	}
}
