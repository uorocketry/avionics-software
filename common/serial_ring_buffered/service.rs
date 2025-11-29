use defmt::info;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::{self, Async};
use embassy_stm32::usart::{Config, Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart};
use embassy_stm32::usart::{ConfigError, RingBufferedUartRx, UartTx};
use embassy_stm32::{Peripheral, usart};
use embassy_time::Timer;
use embedded_io::ReadReady;
use embedded_io_async::{Error, ErrorType, Read, Write};
use heapless::String;
use messages::argus::envelope::Node;
use messages::argus::envelope::{Envelope, envelope::Message as EnvelopeMessage};
use prost::Message;
use utils::serial::traits::{AsyncSerialError, AsyncSerialProvider};

#[derive(Debug)]
pub enum RingBufferedError {
	NoDataAvailable,
	ReadError,
	WriteError,
}

impl Error for RingBufferedError {
	fn kind(&self) -> embedded_io::ErrorKind {
		match self {
			RingBufferedError::NoDataAvailable => embedded_io::ErrorKind::InvalidData,
			RingBufferedError::ReadError => embedded_io::ErrorKind::Other,
			RingBufferedError::WriteError => embedded_io::ErrorKind::Other,
		}
	}
}

pub struct RingBufferedSerialService {
	pub tx_component: UartTx<'static, Async>,
	pub rx_component: RingBufferedUartRx<'static>,
	pub node_type: Node,
}

impl ErrorType for RingBufferedSerialService {
	type Error = RingBufferedError;
}

impl RingBufferedSerialService {
	pub fn new<T: Instance>(
		peri: impl Peripheral<P = T> + 'static,
		tx: impl Peripheral<P = impl TxPin<T>> + 'static,
		rx: impl Peripheral<P = impl RxPin<T>> + 'static,
		interrupt_requests: impl Binding<T::Interrupt, InterruptHandler<T>> + 'static,
		tx_dma: impl Peripheral<P = impl TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
		rx_dma_buff: &'static mut [u8],
		node_type: Node,
		config: Config,
	) -> Result<Self, ConfigError> {
		let uart = Uart::<'static, mode::Async>::new(peri, rx, tx, interrupt_requests, tx_dma, rx_dma, config)?;

		let uart_split = uart.split();

		let tx_component = uart_split.0;
		let mut rx_component = uart_split.1.into_ring_buffered(rx_dma_buff);

		rx_component.start_uart();

		Ok(Self {
			tx_component: tx_component,
			rx_component: rx_component,
			node_type: node_type,
		})
	}

	/// Write the full buffer, waiting until all bytes are sent.
	pub async fn write_all(
		&mut self,
		data: &[u8],
	) -> Result<(), RingBufferedError> {
		match self.tx_component.write_all(data).await {
			Ok(_) => Ok(()),
			Err(_) => Err(RingBufferedError::ReadError),
		}
	}

	/// Convenience helper to write a `&str` fully.
	pub async fn write_str(
		&mut self,
		s: &str,
	) -> Result<(), RingBufferedError> {
		self.write_all(s.as_bytes()).await
	}

	pub async fn read_raw(
		&mut self,
		buff: &mut [u8],
	) -> Result<usize, RingBufferedError> {
		// Checks if the buffer has data (to prevent mutex locks where a read is waiting for data that never comes)
		match self.rx_component.read_ready() {
			Ok(read_ready) => {
				if read_ready {
					match self.rx_component.read(buff).await {
						Ok(len) => {
							return Ok(len);
						}
						Err(_) => {
							return Err(RingBufferedError::ReadError);
						}
					}
				} else {
					return Err(RingBufferedError::NoDataAvailable);
				}
			}
			Err(error) => {
				// Overruns can still contain valid data if the buffer has not been read from for a while (This may induce packet loss)
				if error == usart::Error::Overrun {
					match self.rx_component.read(buff).await {
						Ok(len) => {
							return Ok(len);
						}
						Err(_) => {
							return Err(RingBufferedError::ReadError);
						}
					}
				} else {
					return Err(RingBufferedError::NoDataAvailable);
				}
			}
		}
	}
}

impl AsyncSerialProvider for RingBufferedSerialService {
	async fn read(
		&mut self,
		buff: &mut [u8],
	) -> Result<usize, AsyncSerialError> {
		match self.read_raw(buff).await {
			Ok(len) => return Ok(len),
			Err(error) => match error {
				RingBufferedError::NoDataAvailable => return Err(AsyncSerialError::NoDataAvailable),
				RingBufferedError::ReadError => return Err(AsyncSerialError::ReadError),
				RingBufferedError::WriteError => return Err(AsyncSerialError::WriteError),
			},
		}
	}

	async fn write(
		&mut self,
		data: &[u8],
	) -> Result<(), AsyncSerialError> {
		match self.write_all(data).await {
			Ok(_) => return Ok(()),
			Err(_) => return Err(AsyncSerialError::WriteError),
		}
	}
}

impl AsyncSerialProvider for &mut RingBufferedSerialService {
	async fn read(
		&mut self,
		buff: &mut [u8],
	) -> Result<usize, AsyncSerialError> {
		match self.read_raw(buff).await {
			Ok(len) => return Ok(len),
			Err(error) => match error {
				RingBufferedError::NoDataAvailable => return Err(AsyncSerialError::NoDataAvailable),
				RingBufferedError::ReadError => return Err(AsyncSerialError::ReadError),
				RingBufferedError::WriteError => return Err(AsyncSerialError::WriteError),
			},
		}
	}

	async fn write(
		&mut self,
		data: &[u8],
	) -> Result<(), AsyncSerialError> {
		match self.write_all(data).await {
			Ok(_) => return Ok(()),
			Err(_) => return Err(AsyncSerialError::WriteError),
		}
	}
}
