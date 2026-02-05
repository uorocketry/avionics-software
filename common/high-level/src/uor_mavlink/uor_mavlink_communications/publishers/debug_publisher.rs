use core::array::{self, from_fn};
use core::result::Result::{Err, Ok};

use embassy_executor::task;
use embassy_time::Instant;
use embedded_io_async::{Read, Write};
use uor_proc_macros::Publisher;
use uor_utils::utils::data_structures::ring_buffer::*;
use uor_utils::utils::types::AsyncMutex;

use crate::uor_mavlink::uor_mavlink_communications_traits::publisher::Publisher;
use crate::uor_mavlink::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::DEBUG_DATA};

#[derive(Publisher)]
pub struct DebugDataPublisher<const N: usize> {
	buffer: RingBuffer<DEBUG_DATA, N>,
}
impl<const N: usize> DebugDataPublisher<N> {
	pub fn new() -> DebugDataPublisher<N> {
		return DebugDataPublisher { buffer: RingBuffer::new() };
	}

	pub fn push(
		&mut self,
		mut value: DEBUG_DATA,
	) {
		value.time_boot_ms = Instant::now().as_millis() as u32;
		self.buffer.push(value);
	}
}
