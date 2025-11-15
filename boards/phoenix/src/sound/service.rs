use embassy_stm32::{
	Peripheral,
	gpio::OutputType,
	time::{Hertz, khz},
	timer::{
		Channel1Pin,
		simple_pwm::{PwmPin, SimplePwm},
	},
};
use embassy_time::Timer;

use crate::sound::types::TimerPin;

/// Play sounds using a PWM buzzer.
///
/// Due to there being no equivalent of what `AnyPin` is to GPIO pins for timer pins, the timer
/// picked has to be hard-coded in [TimerPin].
pub struct SoundService {
	pwm: SimplePwm<'static, TimerPin>,
}

impl SoundService {
	pub fn new(
		pin: impl Peripheral<P = TimerPin> + 'static,
		buzzer: impl Peripheral<P = impl Channel1Pin<TimerPin>> + 'static,
	) -> Self {
		let buzz_out_pin = PwmPin::new_ch1(buzzer, OutputType::PushPull);
		let pwm = SimplePwm::new(pin, Some(buzz_out_pin), None, None, None, khz(4), Default::default());
		let mut sound_service = Self { pwm };
		// Set the volume to 100% by default
		sound_service.set_volume(100).unwrap();

		sound_service
	}

	/// Play some pitch with a frequency of `freq` Hz, for `duration` ms.
	pub async fn play_pitch(
		&mut self,
		freq: u32,
		duration: u64,
	) {
		self.pwm.set_frequency(Hertz::hz(freq));
		self.pwm.ch1().enable();
		Timer::after_millis(duration).await;
		self.pwm.ch1().disable();
	}

	/// Set the duty cycle of the buzzer to change output volume.
	///
	/// 100% volume is equal to 50% duty cycle, 0% is 0% duty cycle.
	/// Returns `Err(())` if the value is invalid (> 100). For an unchecked variant prone to
	/// panicking, see [SoundService::set_volume_unchecked]
	pub fn set_volume(
		&mut self,
		volume_percent: u8,
	) -> Result<(), u8> {
		// Technically since volume percentage is half of duty cycle percentage, it's
		// allowable to go up to 200% (Although it makes no real difference)
		if volume_percent > 200 {
			// For now just return the delta in volume
			return Err(volume_percent - 200);
		}

		self.set_volume_unchecked(volume_percent);

		Ok(())
	}

	/// Set the duty cycle of the buzzer to change output volume.
	///
	/// See [SoundService::set_volume] for a checked variant which returns a [Result]
	/// rather than panic on an invalid value.
	pub fn set_volume_unchecked(
		&mut self,
		volume_percent: u8,
	) {
		let duty_cycle_percent = volume_percent / 2;
		self.pwm.ch1().set_duty_cycle_percent(duty_cycle_percent);
	}
}
