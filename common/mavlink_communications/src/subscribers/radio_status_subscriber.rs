use defmt::info;
use mavlink::{MessageData, common::RADIO_STATUS_DATA};
use mavlink_communications_macros::Subscriber;
use utils::data_structures::ring_buffer::RingBuffer;

#[derive(Subscriber)]
pub struct RadioStatusSubscriber<const N: usize> {
	buffer: RingBuffer<RADIO_STATUS_DATA, N>,
}

impl<const N: usize> RadioStatusSubscriber<N> {
	pub fn new() -> RadioStatusSubscriber<N> {
		RadioStatusSubscriber { buffer: RingBuffer::new() }
	}
}
