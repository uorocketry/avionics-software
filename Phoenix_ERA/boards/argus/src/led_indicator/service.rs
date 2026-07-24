use defmt::warn;
use embassy_stm32::gpio::{Level, Output, Pin, Speed};
use embassy_stm32::Peripheral;
use embassy_time::Timer;

pub struct LedIndicatorService<const COUNT: usize> {
	pub pins: [Output<'static>; COUNT],
}

impl<const COUNT: usize> LedIndicatorService<COUNT> {
	pub fn new(peripherals: [impl Peripheral<P = impl Pin> + 'static; COUNT]) -> Self {
		let pins: [Output<'static>; COUNT] = peripherals.map(|pin| {
			let mut output = Output::new(pin, Level::Low, Speed::Low);
			output.set_low();
			output
		});
		Self { pins }
	}

	pub async fn blink(
		&mut self,
		index: usize,
	) {
		if index >= COUNT {
			warn!("LED at an invalid index requested to blink: {}", index);
			return;
		}

		self.pins[index].set_high();
		Timer::after_millis(30).await;
		self.pins[index].set_low();
	}
}
