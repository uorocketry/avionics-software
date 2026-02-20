#[cfg(feature = "embedded")]
use embassy_time::{Delay, Duration, Timer};

#[cfg(feature = "embedded")]
pub struct DelayedPublisher<T> {
	publisher: T,
	delay: Duration,
}
#[cfg(feature = "embedded")]
impl<T> DelayedPublisher<T>
where
	T: Publisher,
{
	pub fn new(
		publisher: T,
		delay: Duration,
	) -> DelayedPublisher<T> {
		DelayedPublisher {
			publisher: publisher,
			delay: delay,
		}
	}

	pub fn internal(&mut self) -> &mut T {
		return &mut self.publisher;
	}
}
#[cfg(feature = "embedded")]
impl<T> Publisher for DelayedPublisher<T>
where
	T: Publisher,
{
	fn publish(
		self: &mut Self,
		buff: &mut [u8],
		sequence: u8,
	) -> usize {
		self.publisher.publish(buff, sequence)
	}
}
#[cfg(feature = "embedded")]
impl<T> Delayed for DelayedPublisher<T> {
	fn get_delay(self: &Self) -> Duration {
		self.delay.clone()
	}
}
#[cfg(feature = "embedded")]
impl<T: Publisher> PeriodicPublisher for DelayedPublisher<T> {}
pub trait Publisher {
	fn publish(
		self: &mut Self,
		buff: &mut [u8],
		sequence: u8,
	) -> usize;
}

#[cfg(feature = "embedded")]
pub trait Delayed {
	fn get_delay(self: &Self) -> Duration;
}

#[cfg(feature = "embedded")]
pub trait PeriodicPublisher: Publisher + Delayed {}
