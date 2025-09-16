#![allow(dead_code)]

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Command {
	// No Operation.
	// Safe placeholder that does nothing — can be used when you need to clock the SPI bus without triggering any action.
	NOP = 0x00,

	// Issues a soft reset of the ADC’s digital core.
	// Equivalent to toggling the RESET pin, but done via SPI. This clears registers to default values.
	RESET = 0x06,

	// Starts ADC1 conversions (the main 32-bit delta-sigma converter).
	// After this, the device begins sampling and toggling DRDY when results are ready.
	START1 = 0x08,

	// Stops ADC1 conversions. The modulator halts, DRDY no longer pulses, and power use is reduced until restarted.
	STOP1 = 0x0A,

	// Reads the latest conversion result from ADC1.
	// You issue this command, then immediately read the output data bytes over SPI.
	RDATA1 = 0x12,
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Register {
	// Device ID register. Lets you confirm you’re talking to an ADS1262 and check silicon revision.
	ID = 0x00,

	// Power and reference control. Controls enabling the internal 2.5 V reference (INTREF), power-down behavior, etc.
	POWER = 0x01,

	// Serial interface options. Can enable/disable appending the status byte, CRC, or watchdog timeout to data frames.
	INTERFACE = 0x02,

	// Conversion mode control. Sets things like chop mode, run/standby, reference reversal, and conversion delay.
	MODE0 = 0x03,

	// Filter and sensor bias. Selects digital filter (Sinc1/2/3/4 or FIR) and optional sensor bias current magnitude/polarity.
	MODE1 = 0x04,

	// Gain and data rate. Configures PGA gain (×1…×32), bypass, and the output data rate.
	MODE2 = 0x05,

	// Input multiplexer. Chooses which analog channel is positive (INP) and which is negative (INN).
	INPMUX = 0x06,

	// Offset calibration registers. Store a 24-bit value to correct zero-offset error.
	OFCAL0 = 0x07,
	OFCAL1 = 0x08,
	OFCAL2 = 0x09,

	// Full-scale calibration registers. Store a 24-bit value to correct gain/scale error.
	FSCAL0 = 0x0A,
	FSCAL1 = 0x0B,
	FSCAL2 = 0x0C,

	// Reference multiplexer register
	REFMUX = 0x0F,
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl AnalogChannel {
	pub fn from(value: u8) -> Self {
		match value {
			0 => AnalogChannel::AIN0,
			1 => AnalogChannel::AIN1,
			2 => AnalogChannel::AIN2,
			3 => AnalogChannel::AIN3,
			4 => AnalogChannel::AIN4,
			5 => AnalogChannel::AIN5,
			6 => AnalogChannel::AIN6,
			7 => AnalogChannel::AIN7,
			8 => AnalogChannel::AIN8,
			9 => AnalogChannel::AIN9,
			10 => AnalogChannel::AINCOM,
			_ => panic!("Invalid AnalogChannel value: {}", value),
		}
	}
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

/// Defines the reference voltage for the ADC.
/// This defines the full-scale-differential input range = VREFP - VREFN / Gain
#[derive(Copy, Clone, Debug)]
pub enum ReferenceRange {
	Avdd,        // REFP = Avdd, REFN = Avss
	Internal2_5, // REFP = Internal 2.5V REFN = Avss
}
impl ReferenceRange {
	pub fn to_volts(&self) -> f32 {
		match self {
			ReferenceRange::Avdd => 5.0,
			ReferenceRange::Internal2_5 => 2.5,
		}
	}
}

/// The 32 bit signed integer value read from the ADC ranges from this negative value to this positive value.
/// This is used to convert the raw ADC code to a voltage.
pub const MAX_SIGNED_CODE_SIZE: f64 = 2147483648.0; // 2^31
