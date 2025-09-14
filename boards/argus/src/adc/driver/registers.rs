#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub
enum Register
{
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
