use embassy_stm32::{
	Peripheral,
	gpio::{Output, Pin},
	mode::Async,
	pac::spi::vals::Master,
	spi::{Error, Spi, Word},
};

use crate::with_cs;

pub struct SPIService<'a> {
	pub internal: Spi<'static, Async>,
	pub cs: Output<'a>,
}

// TODO:Rename this to MonoCsSpiService or smth like that, and create a SPI service that supports multiple CS.
impl<'a> SPIService<'a> {
	pub fn new(
		spi: Spi<'static, Async>,
		cs: impl Peripheral<P = impl Pin> + 'a,
	) -> SPIService<'a> {
		let cs = Output::new(cs, embassy_stm32::gpio::Level::High, embassy_stm32::gpio::Speed::Medium);
		SPIService { internal: spi, cs: cs }
	}

	// SPI write, using DMA.
	pub async fn write<W: Word>(
		&mut self,
		data: &[W],
	) -> Result<(), Error> {
		with_cs!(self, self.internal.write::<W>(data).await)
	}

	// SPI read, using DMA.
	pub async fn read<W: Word>(
		&mut self,
		data: &mut [W],
	) -> Result<(), Error> {
		with_cs!(self, self.internal.read::<W>(data).await)
	}

	// SPI read, using DMA.
	pub async fn read_nocs<W: Word>(
		&mut self,
		data: &mut [W],
	) -> Result<(), Error> {
		self.internal.read::<W>(data).await
	}

	pub async fn write_nocs<W: Word>(
		&mut self,
		data: &[W],
	) -> Result<(), Error> {
		self.internal.write::<W>(data).await
	}

	// Bidirectional transfer, using DMA.

	// This transfers both buffers at the same time, so it is NOT equivalent to write followed by read.
	// The transfer runs for max(read.len(), write.len()) bytes. If read is shorter extra bytes are ignored. If write is shorter it is padded with zero bytes.
	pub async fn transfer<W: Word>(
		&mut self,
		read: &mut [W],
		write: &[W],
	) -> Result<(), Error> {
		with_cs!(self, self.internal.transfer::<W>(read, write).await)
	}
}
