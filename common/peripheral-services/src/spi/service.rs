use embassy_stm32::{
	gpio::Pin,
	mode::Async,
	pac::spi::vals::Master,
	spi::{Error, Spi, Word},
};

pub struct SPIService {
	pub internal: Spi<'static, Async>,
}
// TODO: Add CS, rename this to MonoCsSpiService or smth like that, and create a SPI service that supports multiple CS.
// The former is achievable through a owned buffer of pins and a buffer size generic for the service struct itself.
impl SPIService {
	pub fn new(spi: Spi<'static, Async>) -> SPIService {
		SPIService { internal: spi }
	}

	// SPI write, using DMA.
	pub async fn write<W: Word>(
		&mut self,
		data: &[W],
	) -> Result<(), Error> {
		self.internal.write::<W>(data).await
	}

	// SPI read, using DMA.
	pub async fn read<W: Word>(
		&mut self,
		data: &mut [W],
	) -> Result<(), Error> {
		self.internal.read::<W>(data).await
	}

	// Bidirectional transfer, using DMA.

	// This transfers both buffers at the same time, so it is NOT equivalent to write followed by read.
	// The transfer runs for max(read.len(), write.len()) bytes. If read is shorter extra bytes are ignored. If write is shorter it is padded with zero bytes.
	pub async fn transfer<W: Word>(
		&mut self,
		read: &mut [W],
		write: &[W],
	) -> Result<(), Error> {
		self.internal.transfer::<W>(read, write).await
	}
}
