use defmt::info;
use mavlink::{MessageData, common::DEBUG_DATA};
use mavlink_communications_macros::Subscriber;
use utils::data_structures::ring_buffer::RingBuffer;

#[derive(Subscriber)]
pub struct DebugDataSubscriber<const N: usize> {
	buffer: RingBuffer<DEBUG_DATA, N>,
}

impl<const N: usize> DebugDataSubscriber<N> {
	pub fn new() -> DebugDataSubscriber<N> {
		DebugDataSubscriber { buffer: RingBuffer::new() }
	}
}
