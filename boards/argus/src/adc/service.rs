use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_stm32::Peripheral;
use embassy_stm32::{gpio, mode, spi, time::mhz};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use static_cell::StaticCell;

use crate::adc::driver::Ads1262;

// HACK: Use a static cell to hold the SPI bus shared between multiple ADC instances since we can't have self-referencing structs
// i.e. AdcService holding multiple ADC instances that each hold a reference to the same SPI bus that the ADC service also owns
#[allow(dead_code)]
static ADC_SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, spi::Spi<'static, mode::Async>>> = StaticCell::new();

/// Acts as an orchestration layer for multiple ADC drivers.
/// Variants of this service can be created for different purposes, such as temperature, strain, pressure, etc.
pub struct AdcService<const ADC_COUNT: usize = 2> {
	pub drivers: [AdcDriver; ADC_COUNT],
}

impl<const ADC_COUNT: usize> AdcService<ADC_COUNT> {
	pub fn new<T: spi::Instance>(
		peri: impl Peripheral<P = T> + 'static,
		sck: impl Peripheral<P = impl spi::SckPin<T>> + 'static,
		mosi: impl Peripheral<P = impl spi::MosiPin<T>> + 'static,
		miso: impl Peripheral<P = impl spi::MisoPin<T>> + 'static,
		tx_dma: impl Peripheral<P = impl spi::TxDma<T>> + 'static,
		rx_dma: impl Peripheral<P = impl spi::RxDma<T>> + 'static,
		adc_configs: [AdcConfig; ADC_COUNT],
	) -> Self {
		let mut spi_config = spi::Config::default();
		spi_config.frequency = mhz(8);
		spi_config.mode = spi::MODE_1;

		let spi_bus = ADC_SPI_BUS.init(Mutex::new(spi::Spi::new(peri, sck, mosi, miso, tx_dma, rx_dma, spi_config)));
		let mut adc_configs_iter = adc_configs.into_iter();
		let drivers: [AdcDriver; ADC_COUNT] = core::array::from_fn(|_| {
			let adc_config = adc_configs_iter.next().unwrap();
			let chip_select = gpio::Output::new(adc_config.chip_select, gpio::Level::High, gpio::Speed::VeryHigh);
			let data_ready = gpio::Input::new(adc_config.data_ready, gpio::Pull::None);
			let reset = gpio::Output::new(adc_config.reset, gpio::Level::High, gpio::Speed::VeryHigh);
			let start = gpio::Output::new(adc_config.start, gpio::Level::Low, gpio::Speed::VeryHigh);

			Ads1262::new(SpiDevice::new(spi_bus, chip_select), data_ready, reset, start)
		});

		Self { drivers }
	}
}

/// Config object passed to AdcService for each ADC
pub struct AdcConfig {
	pub chip_select: gpio::AnyPin,
	pub data_ready: gpio::AnyPin,
	pub reset: gpio::AnyPin,
	pub start: gpio::AnyPin,
}

// Type alias for the ADC driver with the specific SPI and GPIO types used within embassy_stm32 instead of embedded_hal
type AdcDriver = Ads1262<
	SpiDevice<'static, CriticalSectionRawMutex, spi::Spi<'static, mode::Async>, gpio::Output<'static>>,
	gpio::Input<'static>,  // Data ready pin (input)
	gpio::Output<'static>, // Reset pin (output)
	gpio::Output<'static>, // Start pin (output)
>;

/// The Spi device error type for the ADC driver
pub type AdcError = embassy_embedded_hal::shared_bus::SpiDeviceError<spi::Error, core::convert::Infallible>;
