use embassy_stm32::peripherals::TIM3;

/// The timer used in [super::service::SoundService](SoundService)
pub type TimerPin = TIM3;
