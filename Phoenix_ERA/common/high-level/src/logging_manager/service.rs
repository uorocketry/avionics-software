use uor_drivers::sd_spi::service::SDCardService;
use uor_utils::{
	buffer_types::rase::pipes::{altitude::AltitudeData, config_data::ConfigData, pose::PoseData, rocket_status::RocketStatusData},
	utils::{
		data_structures::pipe::{Pipe, PipeIn},
		types::AsyncMutex,
	},
};

use crate::uor_mavlink::uor_mavlink_service::service::{MavlinkService, MavlinkServiceTx};
// Right now, mavlink will only be used for transmitting data/frames
pub struct LoggingManager {
	mavlink_service: &'static AsyncMutex<MavlinkService>,
	mavlink_tx: &'static AsyncMutex<MavlinkServiceTx>,
	sd_card: SDCardService,
	pose_pipe: Pipe<PoseData>,
	altitude_pipe: Pipe<AltitudeData>,
	rocket_status_pipe: Pipe<RocketStatusData>,
	configuration_pipe: PipeIn<ConfigData>,
}

impl LoggingManager {
	pub fn new(
		mavlink_service: &'static AsyncMutex<MavlinkService>,
		mavlink_tx: &'static AsyncMutex<MavlinkServiceTx>,
		sd_card: SDCardService,
		pose_pipe: Pipe<PoseData>,
		altitude_pipe: Pipe<AltitudeData>,
		rocket_status_pipe: Pipe<RocketStatusData>,
		configuration_pipe: PipeIn<ConfigData>,
	) -> Self {
		LoggingManager {
			mavlink_service,
			mavlink_tx,
			sd_card,
			pose_pipe,
			altitude_pipe,
			rocket_status_pipe,
			configuration_pipe,
		}
	}

	pub async fn update_mavlink_tx(&mut self) {
		self.mavlink_tx.write_frame(header, payload)
	}
}
