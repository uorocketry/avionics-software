use core::fmt::Write;

use defmt::info;
use embassy_time::{self, Duration, Timer};
use embedded_io_async::ErrorType;
use heapless::String;
use peripheral_services::serial_ring_buffered::service::RingBufferedSerialService;
use peripheral_services::serial_ring_buffered::service::RingBufferedSerialServiceRx;
use peripheral_services::serial_ring_buffered::service::RingBufferedSerialServiceTx;

use crate::in_rfd_ati_mode;
use crate::rfd900x_service::config::*;
use crate::rfd900x_service::data::*;

pub struct RFD900XService {
	io_service: RingBufferedSerialService,
	config: Config,
}

impl RFD900XService {
	pub async fn new(
		io_service: RingBufferedSerialService,
		config: Config,
	) -> RFD900XService {
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
		match self.io_service.tx_component.write(data).await {
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
			.tx_component
			.write(&payload[0..end_of_payload_offset + 2])
			.await
			.expect("Failed to write over uart");
	}

	/// WARNING, ALL CONFIGURATION MUST BE DONE BEFORE THE SPLIT (UNLESS I DECIDE TO PORT THE FUNCTION TO THE RESPECTIVE ELEMENT)
	pub fn split(self) -> (RFD900Tx, RFD900Rx) {
		let io_split = self.io_service.split();
		return (RFD900Tx { component: io_split.0 }, RFD900Rx { component: io_split.1 });
	}
}

impl ErrorType for RFD900XService {
	type Error = RFD900XError;
}

impl embedded_io_async::Read for RFD900XService {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, RFD900XError> {
		// info!("Reached point -1");
		let response = self.io_service.rx_component.read(buf).await;

		match response {
			Ok(len) => return Ok(len),
			Err(_) => return Err(RFD900XError::UsartError),
		}
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<Self::Error>> {
		match self.io_service.read_exact(buf).await {
			Ok(_) => {
				// info!("Read {:?}", buf);
				return Ok(());
			}
			Err(e) => {
				return Err(embedded_io_async::ReadExactError::Other(RFD900XError::ReadError));
			}
		}
	}
}

impl embedded_io_async::Write for RFD900XService {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, RFD900XError> {
		let response = self.io_service.tx_component.write(buf).await;
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

pub struct RFD900Tx {
	pub component: RingBufferedSerialServiceTx,
}

impl ErrorType for RFD900Tx {
	type Error = RFD900XError;
}

impl embedded_io_async::Write for RFD900Tx {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		match self.component.write(buf).await {
			Ok(size) => Ok(size),
			Err(_) => Err(RFD900XError::WriteError),
		}
	}
}

pub struct RFD900Rx {
	pub component: RingBufferedSerialServiceRx,
}

impl ErrorType for RFD900Rx {
	type Error = RFD900XError;
}

impl embedded_io_async::Read for RFD900Rx {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, RFD900XError> {
		// info!("Reached point -1");
		let response = self.component.read(buf).await;

		match response {
			Ok(len) => return Ok(len),
			Err(_) => return Err(RFD900XError::ReadError),
		}
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<RFD900XError>> {
		while !buf.is_empty() {
			match self.component.read_exact(buf).await {
				Ok(_) => {
					return Ok(());
				}
				Err(e) => {
					return Err(embedded_io_async::ReadExactError::Other(RFD900XError::ReadError));
				}
			}
		}
		if buf.is_empty() {
			Ok(())
		} else {
			Err(embedded_io_async::ReadExactError::UnexpectedEof)
		}
	}
}
