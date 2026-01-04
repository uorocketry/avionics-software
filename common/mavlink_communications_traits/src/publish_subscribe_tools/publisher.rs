#[cfg(feature = "delayed")]
use embassy_time::{Delay, Duration, Timer};

// TODO: I forgot why this exists. Look in to why lol
#[cfg(feature = "delayed")]
pub struct DelayedPublisher<T> {
	publisher: T,
	delay: Duration,
}
#[cfg(feature = "delayed")]
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
#[cfg(feature = "delayed")]
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
#[cfg(feature = "delayed")]
impl<T> Delayed for DelayedPublisher<T> {
	fn get_delay(self: &Self) -> Duration {
		self.delay.clone()
	}
}
#[cfg(feature = "delayed")]
impl<T: Publisher> PeriodicPublisher for DelayedPublisher<T> {}
pub trait Publisher {
	fn publish(
		self: &mut Self,
		buff: &mut [u8],
		sequence: u8,
	) -> usize;
}

#[cfg(feature = "delayed")]
pub trait Delayed {
	fn get_delay(self: &Self) -> Duration;
}

#[cfg(feature = "delayed")]
pub trait PeriodicPublisher: Publisher + Delayed {}
