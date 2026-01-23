use defmt::info;
use uor_proc_macros::Subscriber;
use uor_utils::utils::data_structures::ring_buffer::RingBuffer;

use crate::uor_mavlink::uor_mavlink_communications_traits::subscriber::Subscriber;
use crate::uor_mavlink::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::DEBUG_DATA};

// #[derive(Subscriber)]
pub struct DebugDataSubscriber<const N: usize> {
	buffer: RingBuffer<DEBUG_DATA, N>,
}

impl<const N: usize> DebugDataSubscriber<N> {
	pub fn new() -> DebugDataSubscriber<N> {
		DebugDataSubscriber { buffer: RingBuffer::new() }
	}
}
