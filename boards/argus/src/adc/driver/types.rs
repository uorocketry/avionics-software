#![allow(dead_code)]

#[derive(Copy, Clone, Debug)]
pub enum AnalogChannel {
	AIN0 = 0,
	AIN1 = 1,
	AIN2 = 2,
	AIN3 = 3,
	AIN4 = 4,
	AIN5 = 5,
	AIN6 = 6,
	AIN7 = 7,
	AIN8 = 8,
	AIN9 = 9,
	AINCOM = 10,
}

/// Preset Gain values from ADS126x datasheet
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Gain {
	G1 = 0b000,
	G2 = 0b001,
	G4 = 0b010,
	G8 = 0b011,
	G16 = 0b100,
	G32 = 0b101,
}

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

/// Overall data rate of the ADC in samples per second (SPS).
/// Higher data rates give faster response but lower resolution and more noise.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum DataRate {
	Sps2_5 = 0,    // 0b0000,
	Sps5 = 1,      // 0b0001,
	Sps10 = 2,     // 0b0010,
	Sps16_6 = 3,   // 0b0011,
	Sps20 = 4,     // 0b0100,
	Sps50 = 5,     // 0b0101,
	Sps60 = 6,     // 0b0110,
	Sps100 = 7,    // 0b0111,
	Sps400 = 8,    // 0b1000,
	Sps1200 = 9,   // 0b1001,
	Sps2400 = 10,  // 0b1010,
	Sps4800 = 11,  // 0b1011,
	Sps7200 = 12,  // 0b1100,
	Sps14400 = 13, // 0b1101,
	Sps19200 = 14, // 0b1110,
	Sps38400 = 15, // 0b1111,
}

/// Shift the channel's voltage
///
/// MidSupply: shift to (VREFP - VREFN) / 2
/// Useful when using single‑ended sensors (like thermocouples, high‑impedance voltage inputs) that would otherwise float with no defined negative return.
/// Biasing them to mid‑supply keeps the measurement within the ADC’s common‑mode input range.
///
/// None: No shift
/// Useful when using differential sensors (like bridge sensors, 4‑wire RTDs) that already provide a well‑defined return, or if you externally drive the negative terminal.
#[derive(Copy, Clone, Debug)]
pub enum ChannelShift {
	MidSupply,
	None,
}

/// Defines the reference voltage for the ADC.
/// This defines the full-scale-differential input range = VREFP - VREFN / Gain
#[derive(Copy, Clone, Debug)]
pub enum ReferenceVoltageSource {
	Avdd,        // REFP = Avdd, REFN = Avss
	Internal2_5, // REFP = Internal 2.5V REFN = Avss
}
impl ReferenceVoltageSource {
	pub fn to_volts(&self) -> f32 {
		match self {
			ReferenceVoltageSource::Avdd => 5.0,
			ReferenceVoltageSource::Internal2_5 => 2.5,
		}
	}
}

/// The 32 bit signed integer value read from the ADC ranges from this negative value to this positive value.
/// This is used to convert the raw ADC code to a voltage.
pub const MAX_SIGNED_CODE_SIZE: f64 = 2147483648.0; // 2^31
