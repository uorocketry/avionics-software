#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

// #[cfg(not(any(feature = "pressure", feature = "temperature", feature = "strain")))]
// compile_error!(
// 	"You must enable exactly one of the features: 'pressure', 'temperature', or 'strain'."
// );

mod adc;
mod sd;
mod utils;

#[cfg(feature = "temperature")]
mod temperature;

use adc::service::AdcService;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::Pin;
use panic_probe as _;
use sd::service::SDCardService;
use sd::task::{sd_card_task, test_sd_card};
use static_cell::StaticCell;
use utils::hal::configure_hal;
use utils::types::AsyncMutex;

use crate::adc::service::AdcConfig;

// All services are singletons held in a static cell to initialize after peripherals are available
// And wrapped around a mutex so they can be accessed safely from multiple async tasks
static SD_CARD_SERVICE: StaticCell<AsyncMutex<SDCardService>> = StaticCell::new();
static ADC_SERVICE: StaticCell<AsyncMutex<AdcService>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Starting up...");
	let peripherals = configure_hal();
	let sd_card_service = SD_CARD_SERVICE.init(AsyncMutex::new(SDCardService::new(peripherals.SPI1, peripherals.PA5, peripherals.PA7, peripherals.PA6, peripherals.PC4)));
	let adc_service = ADC_SERVICE.init(AsyncMutex::new(AdcService::new(peripherals.SPI4, peripherals.PE2, peripherals.PE6, peripherals.PE5, peripherals.DMA1_CH0, peripherals.DMA1_CH1, [AdcConfig { chip_select: peripherals.PE1.degrade(), data_ready: peripherals.PB9.degrade(), reset: peripherals.PE0.degrade(), start: peripherals.PB0.degrade() }, AdcConfig { chip_select: peripherals.PB8.degrade(), data_ready: peripherals.PB6.degrade(), reset: peripherals.PB7.degrade(), start: peripherals.PB1.degrade() }])));

	#[cfg(feature = "temperature")]
	spawner.must_spawn(temperature::task::temperature_task(adc_service));

	spawner.must_spawn(sd_card_task(sd_card_service));
	spawner.must_spawn(test_sd_card());
}
