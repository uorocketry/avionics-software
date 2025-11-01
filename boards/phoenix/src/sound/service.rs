use embassy_stm32::{
	Peripheral,
	gpio::OutputType,
	time::{Hertz, khz},
	timer::{
		Channel1Pin, GeneralInstance4Channel,
		simple_pwm::{PwmPin, SimplePwm},
	},
};
use embassy_time::Timer;

/// Play sounds using a PWM buzzer.
///
/// This struct requires generics over the timer peripheral used for PWM generation, due to a lack of an `AnyPin` in embassy.
/// This can cause some issues when depending on this service in contexts where generics are not possible (e.g. in an embassy task).
pub struct SoundService<T: GeneralInstance4Channel> {
	pwm: SimplePwm<'static, T>,
}

impl<T: GeneralInstance4Channel> SoundService<T> {
	pub fn new(
		timer: impl Peripheral<P = T> + 'static,
		buzzer: impl Peripheral<P = impl Channel1Pin<T>> + 'static,
	) -> Self {
		let buzz_out_pin = PwmPin::new_ch1(buzzer, OutputType::PushPull);
		let pwm = SimplePwm::new(timer, Some(buzz_out_pin), None, None, None, khz(4), Default::default());
		let mut snd = Self { pwm };
		// Set the volume to 100% by default
		snd.set_volume(100);

		snd
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
	/// The caller is responsible for ensuring that `volume_percent <= 200` (the max
	/// duty cycle).
	pub fn set_volume(
		&mut self,
		volume_percent: u8,
	) {
		// NOTE: I'm not sure on how expensive modulo is on this
		// hardware, so I'll just leave it as is without any protections for
		// going over 100% duty cycle.
		let duty_cycle = volume_percent / 2;
		self.pwm.ch1().set_duty_cycle_percent(duty_cycle);
	}
}
