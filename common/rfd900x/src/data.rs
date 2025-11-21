// DEFUALT CONSTANTS FOR CONFIGURATION
pub const AIR_SPEED_DEFAULT: u8 = 64;
pub const NET_ID_DEFAULT: u8 = 25;
pub const TX_POWER_DEFAULT: u8 = 30;
pub const ECC_DEFUALT: bool = false;
pub const MAVLINK_DEFAULT: bool = true;
pub const OP_RESEND_DEFAULT: bool = false;
pub const MIN_FREQ_DEFAULT: u32 = 915000;
pub const MAX_FREQ_DEFAULT: u32 = 928000;
pub const NUM_OF_CHANNELS_DEFAULT: u8 = 20;
pub const DUTY_CYCLE_DEFAULT: u8 = 100;
pub const LBT_RSSI_DEFAULT: u8 = 0;

// For Point to Point Firmware
pub enum Registers {
	SerialSpeed = 1,
	AirSpeed = 2,
	NetId = 3,
	TxPower = 4,
	ECC = 5,
	Mavlink = 6,
	OpResend = 7,
	MinFreq = 8,
	MaxFreq = 9,
	NumChannels = 10,
	DutyCycle = 11,
	LBTRSSI = 12,
	RTSCTS = 13,
	MaxWindow = 14,
	EncryptionLevel = 15,
	AntMode = 20,
}

pub enum AntMode {
	Diversity = 0,
	Antenna1Only = 1,
	Both = 2,
}
