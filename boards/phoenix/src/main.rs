#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use cortex_m::interrupt;
use defmt::{error, info};
use defmt_rtt as _;
use driver_services::max_m10m_service::{
	message_listeners::{navposllh_listener::NavPosLlhListener, navsat_listener::NavSatListener},
	service::{MaxM10MRx, MaxM10MService, MaxM10MTx, start_maxm10m, start_maxm10m_periodic},
	traits::MaxM10MListener,
};
use embassy_executor::{Spawner, task};
use embassy_stm32::{
	bind_interrupts,
	can::{Frame, frame},
	fmc::DA0Pin,
	peripherals,
	usart::{self, Config},
};
use embassy_time::{Duration, Timer};
use messages::argus::envelope::{Node, NodeType};
use panic_probe as _;
use peripheral_services::{serial::service::SerialService, serial_ring_buffered::service::RingBufferedSerialService};
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
static GPS_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
static GPS_SERVICE: StaticCell<AsyncMutex<MaxM10MService>> = StaticCell::new();
static LISTENERS: StaticCell<[&'static AsyncMutex<dyn MaxM10MListener>; 2]> = StaticCell::new();
static NAV_POS_LISTENR: StaticCell<AsyncMutex<NavPosLlhListener>> = StaticCell::new();
static NAV_SAT_LISTENR: StaticCell<AsyncMutex<NavSatListener>> = StaticCell::new();

bind_interrupts!(struct Irqs {
	UART8 => usart::InterruptHandler<peripherals::UART8>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
	info!("Starting up...");
	let p = configure_hal();
	let mut config: usart::Config = Config::default();
	config.baudrate = 9600;

	let buffer = GPS_BUFFER.init([0; 1024]);

	let serial_service = SerialService::new(
		p.UART8,
		p.PE1,
		p.PE0,
		Irqs,
		p.DMA1_CH0,
		p.DMA1_CH1,
		Node {
			r#type: NodeType::Phoenix as i32,
			id: Some(0),
		},
		config,
	)
	.unwrap();
	let (mut gps_service_temp, mut gps_service_tx, mut gps_service_rx) = MaxM10MService::new(serial_service);
	let gps_service = GPS_SERVICE.init(AsyncMutex::new(gps_service_temp));

	let sound = SOUND_SERVICE.init(AsyncMutex::new(SoundService::new(p.TIM3, p.PC6)));
	gps_service_tx
		.configure(CfgPrtUartBuilder {
			portid: UartPortId::Uart1,
			reserved0: 0,
			tx_ready: 0,
			mode: UartMode::new(DataBits::Eight, Parity::None, StopBits::One),
			baud_rate: 9600,
			in_proto_mask: InProtoMask::UBLOX,
			out_proto_mask: OutProtoMask::union(OutProtoMask::NMEA, OutProtoMask::UBLOX),
			flags: 0,
			reserved5: 0,
		})
		.await;
	let nav_listener = NAV_POS_LISTENR.init(AsyncMutex::new(NavPosLlhListener::new()));
	let sat_listener = NAV_SAT_LISTENR.init(AsyncMutex::new(NavSatListener::new()));
	let listeners = LISTENERS.init([nav_listener, sat_listener]);
	start_maxm10m(&spawner, gps_service, gps_service_rx, listeners, Duration::from_millis(100));
	spawner.spawn(test_write(gps_service_tx));
	spawner.spawn(print_ubx_stream(nav_listener, sat_listener));
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
