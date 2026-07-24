use defmt::{error, info};
use embassy_executor::{SpawnError, Spawner, task};
use embassy_stm32::{
	Peripheral,
	peripherals::IWDG1,
	wdg::{IndependentWatchdog, Instance},
};
use embassy_time::Timer;

// The factor that the timeout is divided by to determine feeding frequency.
// WARNING: Too low of a factor can result in the feeding task not getting priority, resulting in resets.
// For example, with high frequency: SD write -> Feed WDG -> Read Baro -> Feed WDG
// 					 low frequency: SD write -> Read Baro -> Feed WDG (watchdog fed too late, resetting core)
pub const DELAY_FACTOR: u32 = 500;
// TODO: Clean this up and add watchdog minimum
pub fn configure_watchdog(
	mut timeout_us: u32,
	instance: IWDG1,
	task_spawner: &Spawner,
) -> Result<(), SpawnError> {
	if timeout_us < DELAY_FACTOR {
		error!("WATCHDOG TIMEOUT MUST BE AT LEAST THE DELAY FACTOR. DEFAULTING T0 DELAY FACTOR");
		timeout_us = DELAY_FACTOR;
	}

	let watchdog = IndependentWatchdog::new(instance, timeout_us);

	task_spawner.spawn(run_watchdog(watchdog, timeout_us))
}

#[task]
pub async fn run_watchdog(
	mut watchdog: IndependentWatchdog<'static, IWDG1>,
	timeout_us: u32,
) {
	watchdog.unleash();

	loop {
		Timer::after_micros((timeout_us / DELAY_FACTOR) as u64).await;
		watchdog.pet();
	}
}
