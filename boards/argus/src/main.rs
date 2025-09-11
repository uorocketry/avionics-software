#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

#[cfg(not(any(feature = "pressure", feature = "temperature", feature = "strain")))]
compile_error!(
	"You must enable exactly one of the features: 'pressure', 'temperature', or 'strain'."
);

mod utils;
mod sd;
mod adc;
mod _main;

use defmt::debug;
use defmt_rtt as _;
use panic_probe as _;
use embassy_executor::Spawner;

use static_cell::StaticCell;
use heapless::String;
use core::str::FromStr;

use crate::sd::types::OperationScope;
use utils::hal::configure_hal;
use sd::service::SDCardService;
use crate::utils::types::AsyncMutex;

// All services are singletons held in a static cell to initialize after peripherals are available
// And wrapped around a mutex so they can be accessed safely from multiple async tasks
static SD_CARD_SERVICE: StaticCell<AsyncMutex<SDCardService>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	let peripherals = configure_hal();

	let sd_card_service = SD_CARD_SERVICE.init(
		AsyncMutex::new(
				SDCardService::new(
					peripherals.SPI1,
					peripherals.PA5,
					peripherals.PA7,
					peripherals.PA6,
					peripherals.PC4,
				)
		)
	);
	let mut service = sd_card_service.lock().await;
	service.write(OperationScope::Root, String::from_str("test.txt").unwrap(), String::<255>::from_str("This is a test line 1\n").unwrap()).unwrap();
	service.write(OperationScope::Root, String::from_str("test.txt").unwrap(), String::<255>::from_str("This is a test line 2\n").unwrap()).unwrap();
	service.write(OperationScope::Root, String::from_str("test.txt").unwrap(), String::<255>::from_str("This is a test line 3\n").unwrap()).unwrap();
	service.write(OperationScope::Root, String::from_str("test.txt").unwrap(), String::<255>::from_str("This is a test line 4\n").unwrap()).unwrap();
	service.read(OperationScope::Root, String::from_str("test.txt").unwrap(), |line| {
		debug!("Read line from SD card: {:?}", line.as_str());
	}).unwrap();
}

// #[embassy_executor::task]
// async fn test_task() {
// 	for _ in 0..5 {
// 		SDCardService::enqueue_write(OperationScope::CurrentSession, String::from_str("test.txt").unwrap(), String::<255>::from_str("This is a test line ").unwrap()).await;
// 	}
// }
