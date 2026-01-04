use core::fmt::Write;

use defmt::info;
use embassy_time::{self, Duration, Timer};
use embedded_io_async::ErrorType;
use heapless::String;
use utils::serial::traits::AsyncSerialProvider;

use crate::config::*;
use crate::data::*;
use crate::in_rfd_ati_mode;

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

pub struct RFD900XService<T> {
	io_service: T,
	config: Config,
}

impl<T> RFD900XService<T>
where
	T: AsyncSerialProvider,
{
	pub async fn new(
		io_service: T,
		config: Config,
	) -> RFD900XService<T> {
		let mut rfd_service = RFD900XService {
			io_service: io_service,
			config: config,
		};

		in_rfd_ati_mode!(rfd_service, {
			rfd_service.write_config().await.unwrap();
		});
		rfd_service
	}

	/// Write the full buffer, waiting until all bytes are sent.
	pub async fn write_all(
		&mut self,
		data: &[u8],
	) -> Result<(), RFD900XError> {
		match self.io_service.write(data).await {
			Ok(_) => Ok(()),
			Err(_) => Err(RFD900XError::WriteError),
		}
	}

	/// Write a register and wait a bit so the modem can process it
	async fn write_register_with_delay(
		&mut self,
		register_number: Registers,
		value: u32,
	) {
		self.write_register(register_number, value).await;
		Timer::after(Duration::from_millis(CONFIG_WRITE_DELAY_MS)).await;
	}

	/// Writes the internal config to the RFD900
	pub async fn write_config(&mut self) -> Result<(), ConfigurationError> {
		let config: Config = self.config;

		if (!config.check()) {
			return Err(ConfigurationError::InvalidConfig);
		}

		self.write_register_with_delay(Registers::SerialSpeed, config.serial_speed as u32).await;
		self.write_register_with_delay(Registers::AirSpeed, config.air_speed as u32).await;
		self.write_register_with_delay(Registers::NetId, config.net_id as u32).await;
		self.write_register_with_delay(Registers::TxPower, config.tx_power as u32).await;
		self.write_register_with_delay(Registers::ECC, config.ecc as u32).await;
		self.write_register_with_delay(Registers::Mavlink, config.mavlink as u32).await;
		self.write_register_with_delay(Registers::OpResend, config.op_resend as u32).await;
		self.write_register_with_delay(Registers::MinFreq, config.min_freq).await;
		self.write_register_with_delay(Registers::MaxFreq, config.max_freq).await;
		self.write_register_with_delay(Registers::NumChannels, config.num_of_channels as u32)
			.await;
		self.write_register_with_delay(Registers::DutyCycle, config.duty_cycle as u32).await;
		self.write_register_with_delay(Registers::LBTRSSI, config.lbt_rssi as u32).await;
		self.write_register_with_delay(Registers::MaxWindow, config.max_window as u32).await;
		self.write_register_with_delay(Registers::EncryptionLevel, config.encryption_level as u32)
			.await;

		Ok(())
	}

	/// Writes to a register, supports payloads of <= 16 bytes (should not be exceeded when configurating)
	async fn write_register(
		&mut self,
		register_number: Registers,
		value: u32,
	) {
		let mut payload: [u8; MAX_CONFIG_PAYLOAD] = [0; MAX_CONFIG_PAYLOAD];

		payload[0..3].copy_from_slice(b"ATS");

		let register_addressing_offset: usize;
		let end_of_payload_offset: usize;

		let register_digit_offset = number_of_digits(register_number.clone() as u32);
		let value_digit_offset = number_of_digits(value);

		let mut register_component: String<2> = String::new();
		write!(register_component, "{}", register_number as u32);

		let mut value_component: String<8> = String::new();
		write!(value_component, "{}", value as u32);

		payload[ATS_OFFSET..ATS_OFFSET + register_digit_offset].copy_from_slice(register_component.as_bytes());

		payload[ATS_OFFSET + register_digit_offset] = b'=';
		register_addressing_offset = ATS_OFFSET + register_digit_offset + EQUAL_SIGN_OFFSET;

		payload[register_addressing_offset..register_addressing_offset + value_digit_offset].copy_from_slice(value_component.as_bytes());

		let end_of_payload_offset = register_addressing_offset + value_digit_offset;

		payload[end_of_payload_offset..end_of_payload_offset + 2].copy_from_slice(b"\r\n");

		self.io_service
			.write(&payload[0..end_of_payload_offset + 2])
			.await
			.expect("Failed to write over uart");
	}
}

impl<T> ErrorType for RFD900XService<T> {
	type Error = RFD900XError;
}

impl<T> embedded_io_async::Read for RFD900XService<T>
where
	T: AsyncSerialProvider,
{
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, RFD900XError> {
		// info!("Reached point -1");
		let response = self.io_service.read(buf).await;

		match response {
			Ok(len) => return Ok(len),
			Err(_) => return Err(RFD900XError::UsartError),
		}
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<Self::Error>> {
		while !buf.is_empty() {
			match self.read(buf).await {
				Ok(0) => break,
				Ok(n) => buf = &mut buf[n..],
				Err(e) => return Err(embedded_io_async::ReadExactError::Other(RFD900XError::ReadError)),
			}
		}
		if buf.is_empty() {
			Ok(())
		} else {
			Err(embedded_io_async::ReadExactError::UnexpectedEof)
		}
	}
}

impl<T> embedded_io_async::Write for RFD900XService<T>
where
	T: AsyncSerialProvider,
{
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, RFD900XError> {
		let response = self.io_service.write(buf).await;
		match response {
			Ok(_) => return Ok(buf.len()),
			Err(_) => return Err(RFD900XError::UsartError),
		}
	}
}

/// Looks for the number of digits in a u32
pub fn number_of_digits(n: u32) -> usize {
	let mut digits = 1;
	let mut i = n.clone();
	while i >= 10 {
		i /= 10;
		digits += 1;
	}
	digits
}
