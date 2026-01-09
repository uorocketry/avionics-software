use embassy_executor::task;
use embassy_time::Timer;
use mavlink::MAV_STX_V2;
use mavlink_communications_traits::publish_subscribe_tools::publisher::DelayedPublisher;
use mavlink_service::service::{MavlinkService, MavlinkServiceTx};
use rfd900x::service::RFD900XService;
use serial_ring_buffered::service::RingBufferedSerialService;
use utils::types::AsyncMutex;

use mavlink_communications::publishers::heartbeat_publisher::HeartbeatPublisher;

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
