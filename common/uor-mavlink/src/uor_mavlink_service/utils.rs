use driver_services::rfd900x_service::service::RFD900XService;
use embassy_executor::task;
use embassy_time::Timer;
use utils::types::AsyncMutex;

use crate::uor_mavlink_communications::publishers::heartbeat_publisher::HeartbeatPublisher;
use crate::uor_mavlink_communications_traits::publisher::DelayedPublisher;
use crate::uor_mavlink_dialect::MAV_STX_V2;
use crate::uor_mavlink_service::service::{MavlinkService, MavlinkServiceTx};
#[task]
pub async fn update_heartbeat(
	mavlink: &'static AsyncMutex<MavlinkService>,
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
