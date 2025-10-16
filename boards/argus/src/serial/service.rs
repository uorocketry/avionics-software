use embassy_stm32::interrupt::typelevel::Binding;
use embassy_stm32::mode;
use embassy_stm32::usart::ConfigError;
pub use embassy_stm32::usart::Error as UsartError;
use embassy_stm32::usart::{Config, Instance, InterruptHandler, RxDma, RxPin, TxDma, TxPin, Uart};
use embassy_stm32::Peripheral;
use embassy_time::Timer;
use embedded_io_async::Write;
use heapless::String;
use prost;

pub struct SerialService {
	uart: Uart<'static, mode::Async>,
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

	pub async fn write_protobuf<Proto: prost::Message>(
		&mut self,
		message: Proto,
	) -> Result<(), UsartError> {
		let frame = message.encode_length_delimited_to_vec();

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
		let mut count: usize = 0;
		let mut byte = [0u8; 1];
		loop {
			self.uart.read(&mut byte).await?;

			if count >= N {
				// Buffer full: stop reading and return what we have.
				break;
			}

			match byte[0] {
				b'\r' => {
					// Ignore CR to support CRLF or lone CR gracefully.
				}
				b'\n' => {
					// Linefeed terminator: stop and return.
					break;
				}
				b => {
					// Best-effort push; if full, drop additional bytes.
					if out.push(b as char).is_ok() {
						count += 1;
					}
				}
			}
		}
		Ok(count)
	}
}
