pub struct ThroughputAverager {
	tau_secs: f64,
	smoothed_bps: f64,
}

impl ThroughputAverager {
	pub fn new(tau_secs: f64) -> Self {
		Self { tau_secs, smoothed_bps: 0.0 }
	}

	pub fn update(
		&mut self,
		bytes_delta: u64,
		dt_secs: f64,
	) -> f64 {
		let dt = dt_secs.max(1e-3);
		let alpha = 1.0 - (-dt / self.tau_secs).exp();
		let inst = (bytes_delta as f64) / dt;
		self.smoothed_bps = self.smoothed_bps * (1.0 - alpha) + inst * alpha;
		self.smoothed_bps
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ewma_smooths_rate() {
		let mut avg = ThroughputAverager::new(5.0);
		// 1000 bytes per second over 1 second
		let r1 = avg.update(1000, 1.0);
		// next second 0 bytes; smoothed should not drop to zero instantly
		let r2 = avg.update(0, 1.0);
		assert!(r1 > r2);
		assert!(r2 > 0.0);
	}
}
