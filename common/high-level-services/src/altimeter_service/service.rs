use defmt::info;
use driver_services::ms561101::{
	self,
	config::OSR,
	service::MS561101Service,
	units::{Altitude, Pressure, Temperature},
};
use libm::powf;
use uor_utils::utils::data_structures::ring_buffer::RingBuffer;
pub struct AltimeterService<'a> {
	pub barometer: MS561101Service<'a>,
	p0: f64,
	// TODO: Im pretty sure a moving avg filter is a bad solution to the noisy readings, however it works for now
	// TODO: Determine a good buffer size for the filter
	filter: MovingAverageFilter<10>,
}

impl<'a> AltimeterService<'a> {
	pub async fn new(mut barometer: MS561101Service<'a>) -> Self {
		// TODO: p0 should be averaged, rather than a single reading

		let mut p0_filter = MovingAverageFilter::<50>::new();

		for i in 0..50 {
			let pressure = barometer.read_sample(ms561101::config::OSR::OSR4096).await.1.fmbar() as f32;
			p0_filter.push(pressure);
		}

		let p0 = p0_filter.get_average() as f64;
		info!("P0 DETERMINED TO BE: {}", p0);

		let filter = MovingAverageFilter::<10>::new();
		AltimeterService {
			barometer: barometer,
			p0: p0,
			filter: filter,
		}
	}

	// TODO: Return altitude type, not feet repr as f32
	pub async fn altitude(
		&mut self,
		oversampling: OSR,
	) -> Altitude {
		let current_pressure = self.barometer.read_sample(oversampling).await.1.fmbar();

		// Equation takes pressure in mbar and outputs altitude in feet
		let altitude = 145366.45 * (1.0 - powf((current_pressure / self.p0) as f32, 0.190284));

		// info!("\n\nRAW ALTITUDE: {}", altitude);

		self.filter.push(altitude);
		Altitude::new(self.filter.get_average() as f64)
	}

	pub async fn temperature(
		&mut self,
		oversampling: OSR,
	) -> Temperature {
		Temperature::new(self.barometer.read_temperature(&oversampling).await)
	}

	pub async fn pressure(
		&mut self,
		oversampling: OSR,
	) -> Pressure {
		self.barometer.read_sample(oversampling).await.1
	}
}

// TODO: This should be moved to uor-utils/utils
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
