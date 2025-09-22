/// Sinc1, Sinc2, Sinc3, Sinc4 -> Cascaded Sinc (sin(x)/x) filters.
/// Higher order (Sinc4) gives better attenuation of out-of-band noise and higher resolution, but also longer latency and settling time.
/// Lower order (Sinc1) responds faster but passes more noise.
/// FIR -> A fixed FIR filter designed for good rejection of mains interference (50/60 Hz). It gives a balance between noise rejection and throughput.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Filter {
	Sinc1 = 0, // 0b000,
	Sinc2 = 1, // 0b001,
	Sinc3 = 2, // 0b010,
	Sinc4 = 3, // 0b011,
	FIR = 4,   // 0b100,
}
