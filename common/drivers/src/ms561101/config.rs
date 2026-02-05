// All commands contained in this enum do not provide a response. There are additonal commmands, but they have their own respective

#[derive(Default)]
pub struct CalibrationData {
	// Pressure sensitivity
	pub sens: u16,
	// Pressure offset
	pub off: u16,
	// Temperature coefficient of pressure sensitivity
	pub tcs: u16,
	// Temperature coefficient of pressure offset
	pub tco: u16,
	// Reference temperature
	pub tref: u16,
	// Temperature coefficient of the temperature
	pub tempsens: u16,
}

pub enum Commands {
	Reset = 0x1E,
	ADC = 0x00,
	PROM = 0xA0,
}
#[derive(Clone)]

pub enum CalibrationMasks {
	SENS = 0x02,
	OFF = 0x04,
	TCS = 0x06,
	TCO = 0x08,
	TREF = 0x0A,
	TEMPSENS = 0x0C,
	CRC = 0x0E,
}

// Various oversampling rates supported by IC.
// Enums are set to conversion time
/// 256 samples, ~0.60 ms conversion time
/// 512 samples, ~1.17 ms conversion time
/// 1024 samples, ~2.28 ms conversion time
/// 2048 samples, ~4.54 ms conversion time
/// 4096 samples, ~9.04 ms conversion time
#[derive(Clone)]
pub enum OSR {
	OSR256 = 1,
	OSR512 = 2,
	OSR1024 = 3,
	OSR2048 = 5,
	OSR4096 = 10,
}

pub enum SamplingCommands {
	ConvertD1Osr256 = 0x40,
	ConvertD1Osr512 = 0x42,
	ConvertD1Osr1024 = 0x44,
	ConvertD1Osr2048 = 0x46,
	ConvertD1Osr4096 = 0x48,
	ConvertD2Osr256 = 0x50,
	ConvertD2Osr512 = 0x52,
	ConvertD2Osr1024 = 0x54,
	ConvertD2Osr2048 = 0x56,
	ConvertD2Osr4096 = 0x58,
}
