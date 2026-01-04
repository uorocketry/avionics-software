//!  TODO: MUST BE MODIFIED TO UPDATE STATES AND WHATNOT BASED ON MAVLINK
use mavlink::{
	MAVLinkV2MessageRaw, MavHeader,
	common::{HEARTBEAT_DATA, MavAutopilot, MavModeFlag, MavState, MavType},
};
use mavlink_communications_macros::Publisher;
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use utils::{data_structures::ring_buffer::RingBuffer, types::AsyncMutex};

#[derive(Publisher)]
// TODO: Using a generic for this is a band aid solution. Need to fix
pub struct HeartbeatPublisher<const N: usize = 1> {
	buffer: RingBuffer<HEARTBEAT_DATA, N>,
}

impl<const N: usize> HeartbeatPublisher<N> {
	pub fn new() -> HeartbeatPublisher {
		// TODO: This does not work as the heartbeat buffer would empty as it publishes. A work around can be done, but is messy and silly. The derive macro needs to be modified
		let mut ring_buffer = RingBuffer::new();
		HeartbeatPublisher { buffer: ring_buffer }
	}
}
