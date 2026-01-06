use core::array::{self, from_fn};
use core::result::Result::{Err, Ok};

use embassy_executor::task;
use embassy_time::Instant;
use embedded_io_async::{Read, Write};
use mavlink::{MAVLinkV2MessageRaw, MavHeader, common::DEBUG_DATA};
use mavlink_communications_macros::Publisher;
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use utils::data_structures::ring_buffer::*;
use utils::types::AsyncMutex;

#[derive(Publisher)]
pub struct DebugDataPublisher<const N: usize> {
	buffer: RingBuffer<DEBUG_DATA, N>,
}
impl<const N: usize> DebugDataPublisher<N> {
	pub fn new(
		timestamp: u32,
		value: f32,
		index: u8,
	) -> DebugDataPublisher<N> {
		return DebugDataPublisher { buffer: RingBuffer::new() };
	}

	pub fn push(
		&mut self,
		mut value: DEBUG_DATA,
	) {
		// TODO: Check if we want this to be automatic or set by the application using the publisher
		value.time_boot_ms = Instant::now().as_millis() as u32;
		self.buffer.push(value);
	}
}
