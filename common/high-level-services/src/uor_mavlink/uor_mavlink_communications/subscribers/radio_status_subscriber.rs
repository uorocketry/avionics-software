use defmt::info;
use uor_proc_macros::Subscriber;
use uor_utils::utils::data_structures::ring_buffer::RingBuffer;

use crate::uor_mavlink::uor_mavlink_communications_traits::subscriber::Subscriber;
use crate::uor_mavlink::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::RADIO_STATUS_DATA};

#[derive(Subscriber)]
pub struct RadioStatusSubscriber<const N: usize> {
	buffer: RingBuffer<RADIO_STATUS_DATA, N>,
}

impl<const N: usize> RadioStatusSubscriber<N> {
	pub fn new() -> RadioStatusSubscriber<N> {
		RadioStatusSubscriber { buffer: RingBuffer::new() }
	}
}
