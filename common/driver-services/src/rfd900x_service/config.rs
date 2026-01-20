use crate::rfd900x_service::data::EncryptionLevel;

// Delay (in ms) between each configuration command
pub const CONFIG_WRITE_DELAY_MS: u64 = 100;

// Offset to go from a int to a base 10 character
pub const ASCII_NUMBER_OFFSET: u32 = 48;

// Offset past the initial ATS segment in configuring the RFD900
pub const ATS_OFFSET: usize = 3;

// Offset past the equal sign in configuring
pub const EQUAL_SIGN_OFFSET: usize = 1;

pub const MAX_CONFIG_PAYLOAD: usize = 16;

// DEFUALT CONSTANTS FOR CONFIGURATION
pub const SERIAL_SPEED_DEFAULT: u16 = 230;
pub const AIR_SPEED_DEFAULT: u16 = 224;
pub const NET_ID_DEFAULT: u8 = 0;
pub const TX_POWER_DEFAULT: u8 = 30;
pub const ECC_DEFUALT: bool = false;
pub const MAVLINK_DEFAULT: bool = false;
pub const OP_RESEND_DEFAULT: bool = false;
pub const MIN_FREQ_DEFAULT: u32 = 915000;
pub const MAX_FREQ_DEFAULT: u32 = 928000;
pub const NUM_OF_CHANNELS_DEFAULT: u8 = 20;
pub const DUTY_CYCLE_DEFAULT: u8 = 100;
pub const LBT_RSSI_DEFAULT: u8 = 0;
pub const MAX_WINDOW_DEFAULT: u16 = 200;
pub const ENCRYPTION_LEVEL_DEFAULT: EncryptionLevel = EncryptionLevel::Off;

#[derive(Debug)]
pub enum ConfigurationError {
	InvalidConfig,
}

#[derive(Debug)]
pub enum RFD900XError {
	UsartError,
	ReadError,
	WriteError,
}

impl embedded_io_async::Error for RFD900XError {
	fn kind(&self) -> embedded_io_async::ErrorKind {
		embedded_io_async::ErrorKind::Other
	}
}

#[derive(Copy, Clone, Debug, defmt::Format)]
pub struct Config {
	pub serial_speed: u16,
	pub air_speed: u16,
	pub net_id: u8,
	pub tx_power: u8,
	pub ecc: bool,
	pub mavlink: bool,
	pub op_resend: bool,
	pub min_freq: u32,
	pub max_freq: u32,
	pub num_of_channels: u8,
	pub duty_cycle: u8,
	pub lbt_rssi: u8,
	pub max_window: u16,
	pub encryption_level: EncryptionLevel,
}

impl Config {
	/// asserts are there to check that entered values are valid (transmit power isnt too high, frequency isnt too high, etc) according to docs
	pub fn new(
		// Baud rate of uart line
		serial_speed: u16,
		// Air data rate (Needs to be same for both modems)
		air_speed: u16,
		// Network ID. (Needs to be same for both modems)
		net_id: u8,
		// Transmit power in dBm. Maximum is 30dBm (Parameter does not need to be same for both radios)
		tx_power: u8,
		// Enables or disables the Golay error correcting code. When enabled, it doubles the over-the-air data usage (Needs to be same for both modems)
		ecc: bool,
		// Enables or disables the MAVLink framing and reporting (Parameter does not need to be same for both radios)
		mavlink: bool,
		// Opportunistic resend allows the node to resend packets if it has spare bandwidth (Parameter does not need to be same for both radios)
		op_resend: bool,
		// Min frequency in KHz (Needs to be same for both modems)
		min_freq: u32,
		// Max frequency in KHz (Needs to be same for both modems)
		max_freq: u32,
		// Number of frequency hopping channels (Needs to be same for both modems)
		num_of_channels: u8,
		// The percentage of time to allow transmit (Parameter does not need to be same for both radios)
		duty_cycle: u8,
		// Listen before talk threshold (This parameter shouldn’t be changed) (Needs to be same for both modems)
		lbt_rssi: u8,
		// Max transit window size used to limit max time/latency if required otherwise will be set automatically
		max_window: u16,
		// Encryption level
		encryption_level: EncryptionLevel,
	) -> Result<Config, ConfigurationError> {
		let config = Config {
			serial_speed: serial_speed,
			air_speed: air_speed,
			net_id: net_id,
			tx_power: tx_power,
			ecc: ecc,
			mavlink: mavlink,
			op_resend: op_resend,
			min_freq: min_freq,
			max_freq: max_freq,
			num_of_channels: num_of_channels,
			duty_cycle: duty_cycle,
			lbt_rssi: lbt_rssi,
			max_window: max_window,
			encryption_level: encryption_level,
		};
		if (!config.check()) {
			return Err(ConfigurationError::InvalidConfig);
		}
		Ok(config)
	}

	/// Checks a config agaisnt min and max as specified in the datasheet. True indicates that a config is valid
	pub fn check(&self) -> bool {
		if (self.serial_speed < 1 || self.serial_speed > 1000) {
			return false;
		}
		if (self.air_speed < 12 || self.air_speed > 750) {
			return false;
		} else if (self.net_id > 55) {
			return false;
		} else if (self.tx_power > 30) {
			return false;
		} else if (self.min_freq < 902_000 || self.min_freq > 927_000) {
			return false;
		} else if (self.max_freq < 903_000 || self.max_freq > 928_000) {
			return false;
		} else if (self.num_of_channels < 1 || self.num_of_channels > 50) {
			return false;
		} else if (self.duty_cycle < 10 || self.duty_cycle > 100) {
			return false;
		} else if (self.lbt_rssi > 220) {
			return false;
		} else if (self.max_window < 20 || self.max_window > 400) {
			return false;
		}
		return true;
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			serial_speed: SERIAL_SPEED_DEFAULT,
			air_speed: AIR_SPEED_DEFAULT,
			net_id: NET_ID_DEFAULT,
			tx_power: TX_POWER_DEFAULT,
			ecc: ECC_DEFUALT,
			mavlink: MAVLINK_DEFAULT,
			op_resend: OP_RESEND_DEFAULT,
			min_freq: MIN_FREQ_DEFAULT,
			max_freq: MAX_FREQ_DEFAULT,
			num_of_channels: NUM_OF_CHANNELS_DEFAULT,
			duty_cycle: DUTY_CYCLE_DEFAULT,
			lbt_rssi: LBT_RSSI_DEFAULT,
			max_window: MAX_WINDOW_DEFAULT,
			encryption_level: ENCRYPTION_LEVEL_DEFAULT,
		}
	}
}
