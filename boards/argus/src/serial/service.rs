use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode;
use embassy_stm32::usart::ConfigError;
pub use embassy_stm32::usart::Error as UsartError;
use embassy_stm32::usart::{Config, Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart};
use embassy_stm32::Peripheral;
use embassy_time::Timer;
use embedded_io_async::Write;
use heapless::String;
use messages::argus::envelope::{envelope::Message as EnvelopeMessage, Envelope};
use prost::Message;

use crate::node::CURRENT_NODE;

pub struct SerialService {
	pub uart: Uart<'static, mode::Async>,
}

impl SerialService {
	pub fn new<T: Instance>(
		peri: impl Peripheral<P = T> + 'static,
		tx: impl Peripheral<P = impl TxPin<T>> + 'static,
		rx: impl Peripheral<P = impl RxPin<T>> + 'static,
		interrupt_requests: impl Binding<T::Interrupt, InterruptHandler<T>> + 'static,
		tx_dma: impl Peripheral<P = impl TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl RxDma<T>> + 'static,
		baudrate: u32,
	) -> Result<Self, ConfigError> {
		let mut config = Config::default();
		config.baudrate = baudrate;

		let uart = Uart::<'static, mode::Async>::new(peri, rx, tx, interrupt_requests, tx_dma, rx_dma, config)?;

		Ok(Self { uart })
	}

	/// Write the full buffer, waiting until all bytes are sent.
	pub async fn write_all(
		&mut self,
		data: &[u8],
	) -> Result<(), UsartError> {
		self.uart.write_all(data).await
	}

	pub async fn write_envelope_message(
		&mut self,
		message: EnvelopeMessage,
	) -> Result<(), UsartError> {
		let envelope = Envelope {
			created_by: Some(CURRENT_NODE),
			message: Some(message),
		};
		let frame = envelope.encode_length_delimited_to_vec();

		self.uart.write_all(&frame).await?;
		self.uart.flush().await?;

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
			let bytes_size = self.uart.read_until_idle(&mut bytes).await?;
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
}
