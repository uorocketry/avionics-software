use crate::data::EncryptionLevel;

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
