use embedded_io_async::{Error, ErrorType, Read, ReadExactError, Write};
// Trait for read/write providers who function based on a baud rate
pub trait AsyncSerialProvider {
	type Tx: Write;
	type Rx: Read;

	async fn read(
		self: &mut Self,
		buff: &mut [u8],
	) -> Result<usize, AsyncSerialError>;
	async fn write(
		self: &mut Self,
		data: &[u8],
	) -> Result<(), AsyncSerialError>;
	fn split(self: Self) -> (Self::Rx, Self::Tx);
}

#[derive(Debug)]
pub enum AsyncSerialError {
	NoDataAvailable,
	ReadError,
	WriteError,
}

impl Error for AsyncSerialError {
	fn kind(&self) -> embedded_io_async::ErrorKind {
		match self {
			AsyncSerialError::NoDataAvailable => embedded_io_async::ErrorKind::Other,
			AsyncSerialError::ReadError => embedded_io_async::ErrorKind::Other,
			AsyncSerialError::WriteError => embedded_io_async::ErrorKind::Other,
		}
	}
}
