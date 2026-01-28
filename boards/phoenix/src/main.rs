#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use cortex_m::interrupt;
use defmt::{error, info};
use defmt_rtt as _;
use driver_services::{
	ejection_channel_driver::{driver::EjectionChannelDriver, utils::EjectionChannelStates},
	ms561101::service::MS561101Service,
};
use embassy_executor::{Spawner, task};
use embassy_stm32::{
	adc::Temperature,
	bind_interrupts,
	can::{Frame, frame},
	fmc::DA0Pin,
	gpio::Speed,
	peripherals::{self, PA2},
	spi::{self, Mode, Spi},
	time::Hertz,
	usart::{self, Config},
};
use embassy_time::{Duration, Timer};
use high_level_services::altimeter_service::{self, service::AltimeterService};
use panic_probe as _;
use peripheral_services::{serial::service::SerialService, serial_ring_buffered::service::RingBufferedSerialService, spi::service::SPIService};
use phoenix::sound::service::SoundService;
use static_cell::StaticCell;
use uor_utils::utils::types::*;
use uor_utils::utils::{data_structures::ring_buffer::RingBuffer, hal::configure_hal};

/// To change the pin used for sound, see [phoenix::sound::types]
static SOUND_SERVICE: StaticCell<AsyncMutex<SoundService>> = StaticCell::new();
#[cfg(feature = "music")]
static MUSIC_SERVICE: StaticCell<AsyncMutex<phoenix::music::service::MusicService>> = StaticCell::new();
static EJECTION_CHANNEL: StaticCell<AsyncMutex<EjectionChannelDriver>> = StaticCell::new();

bind_interrupts!(struct Irqs {
	UART8 => usart::InterruptHandler<peripherals::UART8>;
});

#[embassy_executor::main]

async fn main(spawner: Spawner) {
	info!("Starting up...");
	let p = configure_hal();

	let detected: Option<PA2> = None;
	let ejection_channel = EJECTION_CHANNEL.init(AsyncMutex::new(EjectionChannelDriver::new(p.PD5, p.PD6, p.PA2, detected)));

	spawner.spawn(ejection_update_process(ejection_channel));
	spawner.spawn(ejection_test_process(ejection_channel));
	#[cfg(feature = "altitude")]
	{
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
		let spi_service = SPIService::new(spi_peripheral, chip_select);

		let mut baro_service = MS561101Service::new(spi_service).await;
		let mut pressure_altimeter_service = AltimeterService::new(baro_service).await;
		spawner.spawn(get_altitude(pressure_altimeter_service));
	}
	let sound = SOUND_SERVICE.init(AsyncMutex::new(SoundService::new(p.TIM3, p.PC6)));
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

#[cfg(feature = "altitude")]
#[task]
pub async fn get_altitude(mut altimeter_service: AltimeterService<'static>) -> ! {
	loop {
		let altitude = altimeter_service.altitude(driver_services::ms561101::config::OSR::OSR4096).await;
		let temperature = altimeter_service.temperature(driver_services::ms561101::config::OSR::OSR4096).await;
		let pressure = altimeter_service.pressure(driver_services::ms561101::config::OSR::OSR4096).await;

		info!("CURRENT ALTITUDE FROM P0: {}m", altitude.fmeters());
		info!("CURRENT TEMPERATURE: {}°C", temperature.fcelsius());
		info!("CURRENT PRESSURE: {}mbar", pressure.fmbar());

		Timer::after_millis(10).await;
	}
}
#[task]

pub async fn ejection_update_process(ejection_channel: &'static AsyncMutex<EjectionChannelDriver<'static>>) {
	let mut last_state: Option<EjectionChannelStates> = None;
	loop {
		{
			ejection_channel.lock().await.update();
			let current_state = &ejection_channel.lock().await.get_state();
			if let Some(state) = &last_state {
				if current_state != state {
					info!("State Transition -- {} -- -> -- {} --", state, current_state);
				}
			} else {
				info!("Initialized -- {} --", current_state);
			}
			last_state = Some(current_state.clone())
		}
		Timer::after_millis(500).await;
	}
}
#[task]

pub async fn ejection_test_process(ejection_channel: &'static AsyncMutex<EjectionChannelDriver<'static>>) {
	loop {
		info!("Ejection channel test starting");
		Timer::after_secs(5).await;
		{
			let mut channel = ejection_channel.lock().await;
			channel.arm();
		}
		Timer::after_secs(5).await;
		{
			let mut channel = ejection_channel.lock().await;
			channel.deploy_charge();
		}
	}
}
