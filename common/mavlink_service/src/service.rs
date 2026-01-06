use core::hint::select_unpredictable;

use defmt::{self, info};
use embassy_executor::{Spawner, task};
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use mavlink::{
	MAVLinkV2MessageRaw, MAX_FRAME_SIZE, MavHeader, MessageData,
	common::{HEARTBEAT_DATA, MavAutopilot, MavComponent, MavMessage, MavMode, MavModeFlag, MavState, MavType, PROTOCOL_VERSION_DATA},
	read_v1_raw_message_async, read_v2_msg_async, read_v2_raw_message_async,
};
use utils::types::AsyncMutex;

use crate::data::SystemId;

#[derive(Debug)]
pub enum MavlinkError {
	TransmitFailed,
	RecieveFailed,
	NoFrameAvailable,
}

pub struct MavlinkService<T> {
	pub io_service: T,
	pub component_id: MavComponent,
	pub mav_type: MavType,
	pub system_id: SystemId,
	pub current_system_mode: MavModeFlag,
	pub current_system_state: MavState,
	pub autopilot: MavAutopilot,

	// The sequence number is u8 according to docs
	internal_sequence: u8,

	pub write_buffer: [u8; MAX_FRAME_SIZE],
}

impl<T> MavlinkService<T>
where
	T: Read + Write,
{
	pub fn new(
		io_service: T,
		component_id: MavComponent,
		mav_type: MavType,
		autopilot: MavAutopilot,
		current_system_mode: MavModeFlag,
		current_system_state: MavState,
		system_id: SystemId,
	) -> MavlinkService<T> {
		MavlinkService {
			io_service: io_service,
			component_id: component_id,
			mav_type: mav_type,
			autopilot: autopilot,
			current_system_mode: current_system_mode,
			current_system_state: current_system_state,
			system_id: system_id,
			internal_sequence: 0,
			write_buffer: [0; MAX_FRAME_SIZE],
		}
	}

	pub async fn write_frame<D>(
		&mut self,
		header: MavHeader,
		payload: D,
	) -> Result<(), MavlinkError>
	where
		D: MessageData, {
		let mut frame = MAVLinkV2MessageRaw::new();
		self.increment_internal_sequence();

		frame.serialize_message_data(header, &payload);
		let result = self.io_service.write(frame.raw_bytes()).await;

		match result {
			Ok(_) => Ok(()),
			Err(_) => Err(MavlinkError::TransmitFailed),
		}
	}

	pub async fn write_internal(
		&mut self,
		num_of_bytes: usize,
	) {
		// info!("Internal Buffer {:?}", self.write_buffer);
		self.io_service.write(&self.write_buffer[0..num_of_bytes]).await;
	}

	pub fn increment_internal_sequence(&mut self) {
		if self.internal_sequence >= 255 {
			self.internal_sequence = 0;
		} else {
			self.internal_sequence += 1;
		}
	}

	pub fn get_internal_sequence(&mut self) -> u8 {
		let current_sequence = self.internal_sequence.clone();
		self.increment_internal_sequence();
		return current_sequence;
	}

	pub async fn read_frame(&mut self) -> Result<MAVLinkV2MessageRaw, MavlinkError> {
		match read_v2_raw_message_async::<MavMessage>(&mut self.io_service).await {
			Ok(frame) => Ok(frame),
			Err(_) => Err(MavlinkError::RecieveFailed),
		}
	}

	pub async fn read_raw(
		&mut self,
		buf: &mut [u8],
	) {
		self.io_service.read(buf).await;
	}

	pub async fn write_raw(
		&mut self,
		buf: &[u8],
	) {
		self.io_service.write(buf).await;
	}

	pub async fn update(&mut self) {
		match self.current_system_state {
			MavState::MAV_STATE_UNINIT => self.set_state(MavState::MAV_STATE_BOOT),
			MavState::MAV_STATE_BOOT => self.set_state(MavState::MAV_STATE_CALIBRATING),
			MavState::MAV_STATE_CALIBRATING => {}
			MavState::MAV_STATE_STANDBY => {}
			MavState::MAV_STATE_ACTIVE => todo!(),
			MavState::MAV_STATE_CRITICAL => todo!(),
			MavState::MAV_STATE_EMERGENCY => todo!(),
			MavState::MAV_STATE_POWEROFF => todo!(),
			MavState::MAV_STATE_FLIGHT_TERMINATION => todo!(),
		}
	}

	// TODO: DelayedPublisher exists, so determine how this should be handled (constantly called, with delay etc) as delays
	// should ideally be handed by the publisher, as they vary in significance

	// pub async fn initialize_publishers(
	// 	&mut self,
	// 	task_spawner: &mut Spawner,
	// ) {
	// 	let mut write_buffer: [u8; mavlink::MAX_FRAME_SIZE] = [0; mavlink::MAX_FRAME_SIZE];

	// 	for i in self.publishers {
	// 		i.lock().await.publish(self.internal_sequence, &mut write_buffer);
	// 		self.increment_internal_sequence();
	// 		self.io_service.write(&write_buffer).await;
	// 	}
	// }

	// /// Returns the number of subscribers who read the frame
	// pub async fn update_subscribers(&mut self) -> Result<usize, MavlinkError> {
	// 	let current_frame;
	// 	let mut number_of_subscriptions = 0;

	// 	match self.read_frame().await {
	// 		Ok(frame) => {
	// 			current_frame = frame;
	// 		}
	// 		Err(_) => return Err(MavlinkError::NoFrameAvailable),
	// 	}

	// 	for i in self.subscribers {
	// 		if i.lock().await.subscribe(current_frame) {
	// 			number_of_subscriptions += 1;
	// 		}
	// 	}
	// 	return Ok(number_of_subscriptions);
	// }

	pub fn set_state(
		&mut self,
		state: MavState,
	) {
		self.current_system_state = state;
	}
}
