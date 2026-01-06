//!  TODO: MUST BE MODIFIED TO UPDATE STATES AND WHATNOT BASED ON MAVLINK
use mavlink::{
	MAV_STX_V2, MAVLinkV2MessageRaw, MavHeader,
	common::{HEARTBEAT_DATA, MavAutopilot, MavModeFlag, MavState, MavType},
};
use mavlink_communications_macros::Publisher;
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
use utils::{data_structures::ring_buffer::RingBuffer, types::AsyncMutex};

#[derive(Publisher)]
#[configure_publisher(is_buffered = false, data_field = internal, override_timestamp = false)]
pub struct HeartbeatPublisher {
	internal: HEARTBEAT_DATA,
}

impl HeartbeatPublisher {
	pub fn new() -> HeartbeatPublisher {
		HeartbeatPublisher {
			internal: HEARTBEAT_DATA {
				custom_mode: 0,
				mavtype: MavType::MAV_TYPE_ROCKET,
				autopilot: MavAutopilot::MAV_AUTOPILOT_GENERIC,
				base_mode: MavModeFlag::DEFAULT,
				system_status: MavState::MAV_STATE_ACTIVE,
				mavlink_version: MAV_STX_V2,
			},
		}
	}

	pub fn update(
		&mut self,
		custom_mode: u32,
		mavtype: MavType,
		autopilot: MavAutopilot,
		base_mode: MavModeFlag,
		system_status: MavState,
		mavlink_version: u8,
	) {
		self.internal = HEARTBEAT_DATA {
			custom_mode: custom_mode,
			mavtype: mavtype,
			autopilot: autopilot,
			base_mode: base_mode,
			system_status: system_status,
			mavlink_version: mavlink_version,
		};
	}
}
