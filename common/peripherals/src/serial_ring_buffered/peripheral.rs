use defmt::info;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::{self, Async};
use embassy_stm32::usart::{Config, Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart};
use embassy_stm32::usart::{ConfigError, RingBufferedUartRx, UartTx};
use embassy_stm32::{Peripheral, usart};
use embedded_io::ReadReady;
use embedded_io_async::{Error, ErrorType, Read, Write};

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

pub struct RingBufferedUORSerial {
	pub tx_component: UartTx<'static, Async>,
	pub rx_component: RingBufferedUartRx<'static>,
}

impl ErrorType for RingBufferedUORSerial {
	type Error = RingBufferedError;
}

impl RingBufferedUORSerial {
	pub fn new<T: Instance>(
		peri: impl Peripheral<P = T> + 'static,
		tx: impl Peripheral<P = impl TxPin<T>> + 'static,
		rx: impl Peripheral<P = impl RxPin<T>> + 'static,
		interrupt_requests: impl Binding<T::Interrupt, InterruptHandler<T>> + 'static,
		tx_dma: impl Peripheral<P = impl TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
		rx_dma_buff: &'static mut [u8],
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
				// Overruns can still contain valid data if the buffer has not been read from for a while (early packets are more likley to be invalid however)
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

	pub fn split(self: Self) -> (RingBufferedUORSerialTx, RingBufferedUORSerialRx) {
		let rx = RingBufferedUORSerialRx {
			component: self.rx_component,
		};
		let tx = RingBufferedUORSerialTx {
			component: self.tx_component,
		};

		return (tx, rx);
	}
}

impl embedded_io_async::Read for RingBufferedUORSerial {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, RingBufferedError> {
		// info!("Reached point -1");
		let response = self.rx_component.read(buf).await;

		match response {
			Ok(len) => return Ok(len),
			Err(_) => return Err(RingBufferedError::ReadError),
		}
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<RingBufferedError>> {
		while !buf.is_empty() {
			match self.rx_component.read_exact(buf).await {
				Ok(_) => {
					return Ok(());
				}
				Err(e) => return Err(embedded_io_async::ReadExactError::Other(RingBufferedError::ReadError)),
			}
		}
		if buf.is_empty() {
			Ok(())
		} else {
			Err(embedded_io_async::ReadExactError::UnexpectedEof)
		}
	}
}

pub struct RingBufferedUORSerialTx {
	pub component: UartTx<'static, Async>,
}

impl ErrorType for RingBufferedUORSerialTx {
	type Error = RingBufferedError;
}

impl embedded_io_async::Write for RingBufferedUORSerialTx {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		match <UartTx<'static, Async> as Write>::write(&mut self.component, buf).await {
			Ok(size) => Ok(size),
			Err(_) => Err(RingBufferedError::WriteError),
		}
	}
}

pub struct RingBufferedUORSerialRx {
	pub component: RingBufferedUartRx<'static>,
}

impl ErrorType for RingBufferedUORSerialRx {
	type Error = RingBufferedError;
}

impl embedded_io_async::Read for RingBufferedUORSerialRx {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, RingBufferedError> {
		// info!("Reached point -1");
		let response = self.component.read(buf).await;

		match response {
			Ok(len) => return Ok(len),
			Err(_) => return Err(RingBufferedError::ReadError),
		}
	}

	async fn read_exact(
		&mut self,
		mut buf: &mut [u8],
	) -> Result<(), embedded_io_async::ReadExactError<RingBufferedError>> {
		while !buf.is_empty() {
			match self.component.read_exact(buf).await {
				Ok(_) => {
					return Ok(());
				}
				Err(e) => return Err(embedded_io_async::ReadExactError::Other(RingBufferedError::ReadError)),
			}
		}
		if buf.is_empty() {
			Ok(())
		} else {
			Err(embedded_io_async::ReadExactError::UnexpectedEof)
		}
	}
}
