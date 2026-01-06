use embassy_executor::task;
use embassy_time::Timer;
use mavlink::MAV_STX_V2;
use mavlink_communications_traits::publish_subscribe_tools::publisher::DelayedPublisher;
use mavlink_service::service::MavlinkService;
use rfd900x::service::RFD900XService;
use serial_ring_buffered::service::RingBufferedSerialService;
use utils::types::AsyncMutex;

use crate::publishers::heartbeat_publisher::HeartbeatPublisher;

pub mod debug_publisher;
pub mod heartbeat_publisher;
pub mod named_value_int_publisher;
// pub mod pressure_sensor_publisher;
// pub mod strain_gauge_publisher;
// pub mod thermocouple_publisher;

// TODO: Move this somewhere
#[task]
pub async fn update_heartbeat(
	mavlink: &'static AsyncMutex<MavlinkService<RFD900XService<&'static mut RingBufferedSerialService>>>,
	heartbeat_publisher: &'static AsyncMutex<DelayedPublisher<HeartbeatPublisher>>,
) {
	loop {
		{
			let mut heartbeat_publisher = heartbeat_publisher.lock().await;
			let mavlink = mavlink.lock().await;

			heartbeat_publisher.internal().update(
				0,
				mavlink.mav_type,
				mavlink.autopilot,
				mavlink.current_system_mode,
				mavlink.current_system_state,
				MAV_STX_V2,
			);
		}
		Timer::after_secs(1).await;
	}
}
