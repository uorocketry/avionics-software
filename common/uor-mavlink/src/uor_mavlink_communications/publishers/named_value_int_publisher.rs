use embassy_time::Instant;
use uor_mavlink_macros::Publisher;
use utils::data_structures::ring_buffer::RingBuffer;

use crate::uor_mavlink_communications_traits::publisher::Publisher;
use crate::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::NAMED_VALUE_INT_DATA, types::CharArray};

#[derive(Publisher)]
pub struct NamedValueIntPublisher<const N: usize> {
	buffer: RingBuffer<NAMED_VALUE_INT_DATA, N>,
}

impl<const N: usize> NamedValueIntPublisher<N> {
	pub fn new(buffer: RingBuffer<NAMED_VALUE_INT_DATA, N>) -> NamedValueIntPublisher<N> {
		NamedValueIntPublisher { buffer: RingBuffer::new() }
	}

	pub fn push(
		&mut self,
		mut value: NAMED_VALUE_INT_DATA,
		index: u8,
	) {
		value.time_boot_ms = Instant::now().as_millis() as u32;
		self.buffer.push(value);
	}
}
