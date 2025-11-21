use defmt::info;
use embassy_stm32::{
	Peripheral,
	can::config,
	interrupt::typelevel::Binding,
	sai::B,
	usart::{Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart},
};
use embassy_time::{self, Duration, Timer};
use heapless::String;
use serial::service::{SerialService, UsartError};

use crate::data::*;
use crate::rfd_ati;

// RFD uses 57600bps for uart transmission
const UART_BAUD: u32 = 57600;

// Delay (in ms) between each configuration command
const CONFIG_WRITE_DELAY_MS: u64 = 100;

// Offset to go from a int to a base 10 character
const ASCII_NUMBER_OFFSET: u32 = 48;

// Offset past the initial ATS segment in configuring the RFD900
const ATS_OFFSET: usize = 3;

// Offset past the equal sign in configuring
const EQUAL_SIGN_OFFSET: usize = 1;

const MAX_CONFIG_PAYLOAD: usize = 16;

pub enum ConfigurationError {
	InvalidConfig,
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
	air_speed: u8,
	net_id: u8,
	tx_power: u8,
	ecc: bool,
	mavlink: bool,
	op_resend: bool,
	min_freq: u32,
	max_freq: u32,
	num_of_channels: u8,
	duty_cycle: u8,
	lbt_rssi: u8,
}

impl Config {
	// asserts are there to check that entered values are valid (transmit power isnt too high, frequency isnt too high, etc) according to docs
	pub fn new(
		// Air data rate (Needs to be same for both modems)
		air_speed: u8,
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
	) -> Result<Config, ConfigurationError> {
		let config = Config {
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
		};
		if (!check_config(config)) {
			return Err(ConfigurationError::InvalidConfig);
		}
		Ok(config)
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
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
		}
	}
}

pub struct RFD900XService {
	uart_service: SerialService,
	config: Config,
}

impl RFD900XService {
	pub async fn new<T: Instance>(
		peri: impl Peripheral<P = T> + 'static,
		tx: impl Peripheral<P = impl TxPin<T>> + 'static,
		rx: impl Peripheral<P = impl RxPin<T>> + 'static,
		interrupt_requests: impl Binding<T::Interrupt, InterruptHandler<T>> + 'static,
		tx_dma: impl Peripheral<P = impl TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
		config: Config,
	) -> RFD900XService {
		let mut uart_service = SerialService::new(peri, tx, rx, interrupt_requests, tx_dma, rx_dma, UART_BAUD);
		let mut rfd_service = RFD900XService {
			uart_service: uart_service.expect("Failed to Initialize UART"),
			config: config,
		};

		rfd_ati!(rfd_service, {
			rfd_service.write_config().await;
		});
		rfd_service
	}

	/// Write the full buffer, waiting until all bytes are sent.
	pub async fn write_all(
		&mut self,
		data: &[u8],
	) -> Result<(), UsartError> {
		self.uart_service.write_all(data).await
	}

	/// Read a single line (LF-terminated). CR bytes are ignored.
	/// Returns the number of bytes pushed into `out` (excluding the terminator).
	pub async fn read_line<const N: usize>(
		&mut self,
		data: &mut String<N>,
	) -> Result<usize, UsartError> {
		self.uart_service.read_line(data).await
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

		if (!check_config(config)) {
			return Err(ConfigurationError::InvalidConfig);
		}

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

		Ok(())
	}

	/// Writes to a register, supports payloads of <= 16 bytes (should not be exceeded when configurating)
	async fn write_register(
		&mut self,
		register_number: Registers,
		value: u32,
	) {
		let mut payload: [u8; MAX_CONFIG_PAYLOAD] = [0; MAX_CONFIG_PAYLOAD];
		payload[0] = b'A';
		payload[1] = b'T';
		payload[2] = b'S';

		let mut register_buffer = [0; 2];
		let mut value_buffer = [0; 8];

		let mut register_addressing_offset = 0;

		let register_digit_offset = prepare_for_register(register_number as u32, &mut register_buffer);
		let value_digit_offset = prepare_for_register(value, &mut value_buffer);

		for i in 0..register_digit_offset {
			payload[ATS_OFFSET + i] = register_buffer[i];
		}

		payload[ATS_OFFSET + register_digit_offset] = b'=';

		register_addressing_offset = ATS_OFFSET + register_digit_offset + EQUAL_SIGN_OFFSET;

		for i in 0..value_digit_offset {
			payload[register_addressing_offset + i] = value_buffer[i];
		}

		payload[register_addressing_offset + value_digit_offset] = b'\r';
		payload[register_addressing_offset + value_digit_offset + 1] = b'\n';

		self.uart_service
			.write_all(&payload[0..ATS_OFFSET + register_digit_offset + EQUAL_SIGN_OFFSET + value_digit_offset + 2])
			.await;
	}
}

/// Looks for the number of digits in a u32
pub fn number_of_digits(mut n: u32) -> usize {
	let mut digits = 1;
	while n >= 10 {
		n /= 10;
		digits += 1;
	}
	digits
}

/// Extracts the ascii representation of a digit in a specific positon, where i is the position in 10^i
fn extract_ascii(
	n: u32,
	i: u32,
) -> u8 {
	let extract_divisor = 10_u32.pow(i);
	(((n / extract_divisor) % 10) + ASCII_NUMBER_OFFSET) as u8
}

/// Converts a u32 into a its ascii form, returns number of digits in array
pub fn prepare_for_register(
	val: u32,
	buffer: &mut [u8],
) -> usize {
	let mut len = number_of_digits(val);

	for i in (0..len).rev() {
		buffer[(len - 1) - i] = extract_ascii(val, i as u32);
	}

	return len;
}

/// Checks a config agaisnt min and max as specified in the datasheet. True indicates that a config is valid
pub fn check_config(config: Config) -> bool {
	if (config.air_speed < 12 || config.air_speed > 250) {
		return false;
	} else if (config.net_id > 55) {
		return false;
	} else if (config.tx_power > 30) {
		return false;
	} else if (config.min_freq < 902_000 || config.min_freq > 927_000) {
		return false;
	} else if (config.max_freq < 903_000 || config.max_freq > 928_000) {
		return false;
	} else if (config.num_of_channels < 1 || config.num_of_channels > 50) {
		return false;
	} else if (config.duty_cycle < 10 || config.duty_cycle > 100) {
		return false;
	} else if (config.lbt_rssi > 220) {
		return false;
	}
	return true;
}
