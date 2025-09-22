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
