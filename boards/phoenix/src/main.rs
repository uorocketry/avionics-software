#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use cortex_m::interrupt;
use defmt::{error, info};
use defmt_rtt as _;
use driver_services::{
	max_m10m_service::{
		message_listeners::{navposllh_listener::NavPosLlhListener, navsat_listener::NavSatListener},
		service::{MaxM10MRx, MaxM10MService, MaxM10MTx, start_maxm10m, start_maxm10m_periodic},
		traits::MaxM10MListener,
	},
	ms561101::service::MS561101Service,
};
use embassy_executor::{Spawner, task};
use embassy_stm32::{
	bind_interrupts,
	can::{Frame, frame},
	fmc::DA0Pin,
	gpio::Speed,
	peripherals,
	spi::Spi,
	spi::{self, Mode},
	time::Hertz,
	usart::{self, Config},
};
use embassy_time::{Duration, Timer};
use messages::argus::{
	envelope::{Node, NodeType},
	pressure,
};
use panic_probe as _;
use peripheral_services::{serial::service::SerialService, serial_ring_buffered::service::RingBufferedSerialService, spi::service::SPIService};
use phoenix::sound::service::SoundService;
use static_cell::StaticCell;
use ublox::{
	self, FixedBuffer, Parser, ParserBuilder,
	cfg_msg::CfgMsgSinglePort,
	cfg_prt::CfgPrtUartBuilder,
	mon_gnss::{MonGnss, MonGnssConstellMask},
	mon_rf::MonRf,
	nav_pos_llh::NavPosLlh,
	nav_sat::NavSat,
};
use ublox::{
	cfg_prt::{DataBits, InProtoMask, OutProtoMask, Parity, StopBits, UartMode, UartPortId},
	mon_ver::MonVer,
};
use utils::hal::configure_hal;
use utils::types::*;
/// To change the pin used for sound, see [phoenix::sound::types]
static SOUND_SERVICE: StaticCell<AsyncMutex<SoundService>> = StaticCell::new();
#[cfg(feature = "music")]
static MUSIC_SERVICE: StaticCell<AsyncMutex<phoenix::music::service::MusicService>> = StaticCell::new();

bind_interrupts!(struct Irqs {
	UART8 => usart::InterruptHandler<peripherals::UART8>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Starting up...");
	let p = configure_hal();
	let chip_select = p.PB8;
	let mut spi_config = embassy_stm32::spi::Config::default();
	spi_config.mode = Mode {
		polarity: spi::Polarity::IdleLow,
		phase: spi::Phase::CaptureOnFirstTransition,
	};
	spi_config.bit_order = spi::BitOrder::MsbFirst;
	spi_config.frequency = Hertz::khz(1);
	spi_config.miso_pull = embassy_stm32::gpio::Pull::Down;
	spi_config.rise_fall_speed = Speed::Low;

	let spi_peripheral = Spi::new(p.SPI4, p.PE2, p.PE6, p.PE5, p.DMA1_CH0, p.DMA1_CH1, spi_config);
	let spi_service = SPIService::new(spi_peripheral);

	let mut baro_service = MS561101Service::new(spi_service, chip_select).await;

	let sound = SOUND_SERVICE.init(AsyncMutex::new(SoundService::new(p.TIM3, p.PC6)));
	spawner.spawn(baro_test(baro_service));
	#[cfg(feature = "music")]
	{
		use defmt::error;
		use phoenix::music::{service::MusicService, tasks::play_music_forever};
		let music = MUSIC_SERVICE.init(AsyncMutex::new(MusicService::new(sound)));
		match spawner.spawn(play_music_forever(music)) {
			Ok(_) => (),
			Err(e) => error!("Could not spawn music task: {}", e),
		}
	}
}

#[task]
pub async fn baro_test(mut baro_service: MS561101Service<'static>) -> ! {
	loop {
		let (temp, pressure) = baro_service.read_sample(driver_services::ms561101::config::OSR::OSR1024).await;
		// let pressure = baro_service.read_pressure_raw(&driver_services::ms561101::config::OSR::OSR256).await;

		info!("Read temperature from barometer: {}", temp.fcelsius());
		info!("Read pressure from barometer: {}", pressure.fbar());
		Timer::after_millis(50).await;
	}
}

#[task]
pub async fn print_ubx_stream(
	data: &'static AsyncMutex<NavPosLlhListener>,
	data_2: &'static AsyncMutex<NavSatListener>,
) -> ! {
	info!("Starting read");
	loop {
		{
			// Everything but the logging should be wrapped in a function
			let mut data = data.lock().await;
			if let Some(payload) = data.internal.clone() {
				info!("CURRENT DATA LAT: {}", payload.lat_degrees());
				info!("CURRENT DATA LON: {}", payload.lon_degrees());
				info!("CURRENT DATA FRESH: {}", data.new_data);
			} else {
				info!("No data available");
			}
			data.new_data = false;
		}
		{
			// Everything but the logging should be wrapped in a function
			let mut data_2 = data_2.lock().await;
			if let Some(payload) = data_2.internal.clone() {
				info!("CURRENT SATELLITE COUNT: {}", payload.num_svs());
				info!("CURRENT DATA FRESH: {}", data_2.new_data);
			} else {
				info!("No data available");
			}
			data_2.new_data = false;
		}
		Timer::after_millis(250).await;
	}
}
#[task]
pub async fn test_write(mut service: MaxM10MTx) -> ! {
	loop {
		service.update_message::<NavPosLlh>().await;
		service.update_message::<NavSat>().await;
		// service.update_message::<NavPosLlh>().await;

		Timer::after_millis(100).await;
		// info!("Writing");
	}
}
