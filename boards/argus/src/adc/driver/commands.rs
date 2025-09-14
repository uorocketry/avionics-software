#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub
enum Command
{
	// No Operation.
	// Safe placeholder that does nothing — can be used when you need to clock the SPI bus without triggering any action.
	NOP	= 0x00,

	// Issues a soft reset of the ADC’s digital core. 
	// Equivalent to toggling the RESET pin, but done via SPI. This clears registers to default values.
	RESET  = 0x06,

	// Starts ADC1 conversions (the main 32-bit delta-sigma converter). 
	// After this, the device begins sampling and toggling DRDY when results are ready.
	START1 = 0x08,

	// Stops ADC1 conversions. The modulator halts, DRDY no longer pulses, and power use is reduced until restarted.
	STOP1  = 0x0A,

	// Reads the latest conversion result from ADC1. 
	// You issue this command, then immediately read the output data bytes over SPI.
	RDATA1 = 0x12,
}
