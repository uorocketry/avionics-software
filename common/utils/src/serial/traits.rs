// Trait for read/write providers who function based on a baud rate
pub trait AsyncSerialProvider {
	async fn read(
		self: &mut Self,
		buff: &mut [u8],
	) -> Result<usize, AsyncSerialError> {
		Ok(0)
	}
	async fn write(
		self: &mut Self,
		data: &[u8],
	) -> Result<(), AsyncSerialError> {
		Ok(())
	}
}

// Errors for the AsyncSerialProvider trait
#[derive(Debug)]
pub enum AsyncSerialError {
	NoDataAvailable,
	ReadError,
	WriteError,
}
