use core::mem::swap;

use defmt::info;
use embassy_stm32::Peripheral;
use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode::{self, Async};
pub use embassy_stm32::usart::Error as UsartError;
use embassy_stm32::usart::{Config, Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart};
use embassy_stm32::usart::{ConfigError, UartRx, UartTx};
use embassy_time::Timer;
use embedded_io_async::{ErrorType, Read, Write};
use heapless::String;
#[cfg(feature = "messages")]
use prost::Message;
#[cfg(feature = "messages")]
use uor_utils::messages::argus::envelope::{Envelope, Node, envelope::Message as EnvelopeMessage};

pub struct SerialService {
	pub tx_component: SerialServiceTx,
	pub rx_component: SerialServiceRx,
}

impl SerialService {
	pub fn new<T: Instance>(
		peri: impl Peripheral<P = T> + 'static,
		tx: impl Peripheral<P = impl TxPin<T>> + 'static,
		rx: impl Peripheral<P = impl RxPin<T>> + 'static,
		interrupt_requests: impl Binding<T::Interrupt, InterruptHandler<T>> + 'static,
		tx_dma: impl Peripheral<P = impl TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
		config: Config,
	) -> Result<Self, ConfigError> {
		let uart = Uart::<'static, mode::Async>::new(peri, rx, tx, interrupt_requests, tx_dma, rx_dma, config)?;
		let (tx_component, rx_component) = uart.split();

		Ok(Self {
			tx_component: SerialServiceTx { component: tx_component },
			rx_component: SerialServiceRx { component: rx_component },
		})
	}

	/// Write the full buffer, waiting until all bytes are sent.
	pub async fn write_all(
		&mut self,
		data: &[u8],
	) -> Result<(), UsartError> {
		self.tx_component.component.write_all(data).await
	}

	#[cfg(feature = "messages")]
	pub async fn write_envelope_message(
		&mut self,
		message: EnvelopeMessage,
	) -> Result<(), UsartError> {
		let envelope = Envelope {
			// TODO: The Node::default should be corrected to how it was in the legacy argus system, but this is low priority RN
			created_by: Some(Node::default()),
			message: Some(message),
		};
		let frame = envelope.encode_length_delimited_to_vec();

		self.tx_component.component.write_all(&frame).await?;
		self.tx_component.component.flush().await?;

		Timer::after_millis(100).await;
		Ok(())
	}

	/// Convenience helper to write a `&str` fully.
	pub async fn write_str(
		&mut self,
		s: &str,
	) -> Result<(), UsartError> {
		self.write_all(s.as_bytes()).await
	}

	/// Read a single line (LF-terminated). CR bytes are ignored.
	/// Returns the number of bytes pushed into `out` (excluding the terminator).
	pub async fn read_line<const N: usize>(
		&mut self,
		out: &mut String<N>,
	) -> Result<usize, UsartError> {
		out.clear();
		let mut count: usize = 0;
		let mut bytes = [0u8; 32];

		'chunking_loop: loop {
			let bytes_size = self.rx_component.component.read_until_idle(&mut bytes).await?;
			for byte in &bytes[..bytes_size] {
				if count >= N {
					// Buffer full: stop reading and return what we have.
					break 'chunking_loop;
				}

				match byte {
					b'\r' => {
						// Ignore CR to support CRLF or lone CR gracefully.
					}
					b'\n' => {
						// Linefeed terminator: stop and return.
						break 'chunking_loop;
					}
					b => {
						// Best-effort push; if full, drop additional bytes.
						if out.push(*b as char).is_ok() {
							count += 1;
						}
					}
				}
			}
		}
		Ok(count)
	}

	pub async fn read_raw(
		&mut self,
		buff: &mut [u8],
	) -> Result<usize, UsartError> {
		match self.rx_component.component.read_until_idle(buff).await {
			Ok(len) => Ok(len),
			Err(error) => Err(error),
		}
	}

	pub async fn read(
		&mut self,
		buf: &mut [u8],
	) {
		self.rx_component.component.read_until_idle(buf).await;
	}

	pub fn split(self) -> (SerialServiceTx, SerialServiceRx) {
		return (self.tx_component, self.rx_component);
	}
}

pub struct SerialServiceTx {
	pub component: UartTx<'static, Async>,
}

impl ErrorType for SerialServiceTx {
	type Error = embassy_stm32::usart::Error;
}
impl Write for SerialServiceTx {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		<UartTx<'static, Async> as embedded_io_async::Write>::write(&mut self.component, buf).await
	}
}

pub struct SerialServiceRx {
	pub component: UartRx<'static, Async>,
}
impl ErrorType for SerialServiceRx {
	type Error = embassy_stm32::usart::Error;
}
impl Read for SerialServiceRx {
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, Self::Error> {
		self.component.read_until_idle(buf).await
	}
}
