/// IIM-20670 6-Axis IMU Configuration

// Protocol Constants
/// Fixed value register content (Bank 0, offset 0x0B)
pub const FIXED_VALUE: u16 = 0xAA55;

/// CRC seed value for the 32-bit SPI protocol
pub const CRC_SEED: u8 = 0xFF;

// Register Offsets (5-bit address within Bank 0)
pub mod reg {
	// Sensor output data (available in all banks)
	pub const GYRO_X_DATA: u8 = 0x00;
	pub const GYRO_Y_DATA: u8 = 0x01;
	pub const GYRO_Z_DATA: u8 = 0x02;
	pub const TEMP1_DATA: u8 = 0x03;
	pub const ACCEL_X_DATA: u8 = 0x04;
	pub const ACCEL_Y_DATA: u8 = 0x05;
	pub const ACCEL_Z_DATA: u8 = 0x06;
	pub const TEMP2_DATA: u8 = 0x07;

	// Bank 0 only
	pub const ACCEL_X_DATA_LR: u8 = 0x08;
	pub const ACCEL_Y_DATA_LR: u8 = 0x09;
	pub const ACCEL_Z_DATA_LR: u8 = 0x0A;
	pub const FIXED_VALUE: u8 = 0x0B;
	pub const RESET_CTRL: u8 = 0x18;
}

// Gyroscope Full-Scale Range (Table 18)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GyroFsSel {
	/// ±328 dps, 100 LSB/dps
	Dps328_0 = 0b0000,
	/// ±655 dps, 50 LSB/dps (DEFAULT)
	Dps655 = 0b0001,
	/// ±1311 dps, 25 LSB/dps
	Dps1311_0 = 0b0010,
	/// ±1966 dps, 16.67 LSB/dps
	Dps1966 = 0b0011,
	/// ±218 dps, 150 LSB/dps
	Dps218 = 0b0100,
	/// ±437 dps, 75 LSB/dps
	Dps437 = 0b0101,
	/// ±874 dps, 37.5 LSB/dps
	Dps874 = 0b0110,
	/// ±1311 dps, 25 LSB/dps
	Dps1311_1 = 0b0111,
	/// ±61 dps, 533.34 LSB/dps
	Dps61 = 0b1000,
	/// ±123 dps, 266.67 LSB/dps
	Dps123 = 0b1001,
	/// ±246 dps, 133.33 LSB/dps
	Dps246 = 0b1010,
	/// ±492 dps, 66.67 LSB/dps
	Dps492 = 0b1011,
	/// ±41 dps, 800 LSB/dps
	Dps41 = 0b1100,
	/// ±82 dps, 400 LSB/dps
	Dps82 = 0b1101,
	/// ±164 dps, 200 LSB/dps
	Dps164 = 0b1110,
	/// ±328 dps, 100 LSB/dps
	Dps328_1 = 0b1111,
}

impl GyroFsSel {
	/// Returns the full-scale range in dps (degrees per second)
	pub fn full_scale_dps(&self) -> f32 {
		match self {
			Self::Dps328_0 | Self::Dps328_1 => 328.0,
			Self::Dps655 => 655.36,
			Self::Dps1311_0 | Self::Dps1311_1 => 1311.0,
			Self::Dps1966 => 1966.0,
			Self::Dps218 => 218.0,
			Self::Dps437 => 437.0,
			Self::Dps874 => 874.0,
			Self::Dps61 => 61.0,
			Self::Dps123 => 123.0,
			Self::Dps246 => 246.0,
			Self::Dps492 => 492.0,
			Self::Dps41 => 41.0,
			Self::Dps82 => 82.0,
			Self::Dps164 => 164.0,
		}
	}

	/// Returns the sensitivity in LSB/dps
	pub fn sensitivity(&self) -> f32 {
		match self {
			Self::Dps328_0 | Self::Dps328_1 => 100.0,
			Self::Dps655 => 50.0,
			Self::Dps1311_0 | Self::Dps1311_1 => 25.0,
			Self::Dps1966 => 16.67,
			Self::Dps218 => 150.0,
			Self::Dps437 => 75.0,
			Self::Dps874 => 37.5,
			Self::Dps61 => 533.34,
			Self::Dps123 => 266.67,
			Self::Dps246 => 133.33,
			Self::Dps492 => 66.67,
			Self::Dps41 => 800.0,
			Self::Dps82 => 400.0,
			Self::Dps164 => 200.0,
		}
	}
}

impl Default for GyroFsSel {
	fn default() -> Self {
		Self::Dps655
	}
}

// Accelerometer Full-Scale Range (Table 17)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccelFsSel {
	/// FS = ±16.384g, FS_LR = ±32.768g
	G16Lr32 = 0b000,
	/// FS = ±16.384g, FS_LR = ±65.536g (DEFAULT)
	G16Lr65 = 0b001,
	/// FS = ±32.768g, FS_LR = ±32.768g
	G32Lr32 = 0b010,
	/// FS = ±32.768g, FS_LR = ±65.536g
	G32Lr65 = 0b011,
	/// FS = ±2.048g, FS_LR = ±4.096g
	G2Lr4 = 0b100,
	/// FS = ±2.048g, FS_LR = ±16.384g
	G2Lr16 = 0b101,
	/// FS = ±4.096g, FS_LR = ±4.096g
	G4Lr4 = 0b110,
	/// FS = ±4.096g, FS_LR = ±8.192g
	G4Lr8 = 0b111,
}

impl AccelFsSel {
	/// Returns the high-resolution full-scale range in g
	pub fn full_scale_g(&self) -> f32 {
		match self {
			Self::G16Lr32 | Self::G16Lr65 => 16.384,
			Self::G32Lr32 | Self::G32Lr65 => 32.768,
			Self::G2Lr4 | Self::G2Lr16 => 2.048,
			Self::G4Lr4 | Self::G4Lr8 => 4.096,
		}
	}

	/// Returns the low-resolution full-scale range in g
	pub fn full_scale_lr_g(&self) -> f32 {
		match self {
			Self::G16Lr32 => 32.768,
			Self::G16Lr65 => 65.536,
			Self::G32Lr32 => 32.768,
			Self::G32Lr65 => 65.536,
			Self::G2Lr4 => 4.096,
			Self::G2Lr16 => 16.384,
			Self::G4Lr4 => 4.096,
			Self::G4Lr8 => 8.192,
		}
	}
}

impl Default for AccelFsSel {
	fn default() -> Self {
		Self::G16Lr65
	}
}

// SPI Return Status
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpiReturnStatus {
	/// 00: Reserved
	Reserved,
	/// 01: Successful register read/write
	Success,
	/// 10: Data not ready or self-test in progress
	InProgress,
	/// 11: Error
	Error,
}

impl From<u8> for SpiReturnStatus {
	fn from(val: u8) -> Self {
		match val & 0x03 {
			0b00 => Self::Reserved,
			0b01 => Self::Success,
			0b10 => Self::InProgress,
			0b11 => Self::Error,
			_ => unreachable!(),
		}
	}
}

// CRC Calculation (Section 5.2 of datasheet)
/// The result is inverted before returning (per datasheet).
pub fn compute_crc(input_24bit: u32) -> u8 {
	let mut crc: u8 = CRC_SEED;

	for i in (0..24).rev() {
		let bit = ((input_24bit >> i) & 1) as u8;
		let msb = (crc >> 7) & 1;

		let mut new_crc: u8 = 0;
		new_crc |= (bit ^ msb) << 0; // CRC_New[0]
		new_crc |= (crc & 0x01) << 1; // CRC_New[1] = CRC[0]
		new_crc |= ((crc >> 1) ^ msb) << 2; // CRC_New[2] = CRC[1] ^ CRC[7]
		new_crc |= ((crc >> 2) ^ msb) << 3; // CRC_New[3] = CRC[2] ^ CRC[7]
		new_crc |= ((crc >> 3) ^ msb) << 4; // CRC_New[4] = CRC[3] ^ CRC[7]
		new_crc |= ((crc >> 4) & 1) << 5; // CRC_New[5] = CRC[4]
		new_crc |= ((crc >> 5) & 1) << 6; // CRC_New[6] = CRC[5]
		new_crc |= ((crc >> 6) & 1) << 7; // CRC_New[7] = CRC[6]

		crc = new_crc;
	}

	// Final inversion
	crc ^ 0xFF
}

pub fn build_spi_command(
	read: bool,
	offset: u8,
	data: u16,
) -> u32 {
	let rw_bit: u32 = if read { 0 } else { 1 };
	let addr = (offset & 0x1F) as u32;

	// Upper 24 bits: RW(1) | ADDR(5) | RS(2, always 00 on MOSI) | DATA(16)
	let upper_24 = (rw_bit << 23) | (addr << 18) | (data as u32);
	let crc = compute_crc(upper_24);

	(upper_24 << 8) | (crc as u32)
}

pub fn parse_spi_response(response: u32) -> (u8, SpiReturnStatus, u16, bool) {
	let offset = ((response >> 26) & 0x1F) as u8;
	let status = SpiReturnStatus::from(((response >> 24) & 0x03) as u8);
	let data = ((response >> 8) & 0xFFFF) as u16;
	let received_crc = (response & 0xFF) as u8;

	// Verify CRC over the upper 24 bits of the response
	let upper_24 = (response >> 8) & 0x00FF_FFFF;
	let computed_crc = compute_crc(upper_24);
	let crc_valid = received_crc == computed_crc;

	(offset, status, data, crc_valid)
}
