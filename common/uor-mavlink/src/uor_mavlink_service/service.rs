use defmt::{self, info};
use driver_services::rfd900x::service::{RFD900Rx, RFD900Tx, RFD900XService};
use embassy_executor::{Spawner, task};
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};

use crate::uor_mavlink_dialect::{
	MAV_STX, MAVLinkV1MessageRaw, MAVLinkV2MessageRaw, MAX_FRAME_SIZE, MavHeader, MessageData,
	common::{HEARTBEAT_DATA, MavAutopilot, MavComponent, MavMessage, MavMode, MavModeFlag, MavState, MavType},
	read_v1_raw_message_async, read_v2_raw_message_async,
};
use crate::uor_mavlink_service::data::SystemId;

#[derive(Debug)]
pub enum MavlinkError {
	TransmitFailed,
	ReceiveFailed,
	NoFrameAvailable,
}

pub struct MavlinkService {
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

impl MavlinkService {
	pub fn new(
		io_service: RFD900XService,
		component_id: MavComponent,
		mav_type: MavType,
		autopilot: MavAutopilot,
		current_system_mode: MavModeFlag,
		current_system_state: MavState,
		system_id: SystemId,
	) -> (MavlinkService, MavlinkServiceTx, MavlinkServiceRx) {
		let io_service = io_service.split();

		(
			MavlinkService {
				component_id: component_id,
				mav_type: mav_type,
				autopilot: autopilot,
				current_system_mode: current_system_mode,
				current_system_state: current_system_state,
				system_id: system_id,
				internal_sequence: 0,
				write_buffer: [0; MAX_FRAME_SIZE],
			},
			MavlinkServiceTx::new(io_service.0),
			MavlinkServiceRx::new(io_service.1),
		)
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

	pub fn set_state(
		&mut self,
		state: MavState,
	) {
		self.current_system_state = state;
	}
}
pub struct MavlinkServiceTx {
	pub io_service: RFD900Tx,
	// The sequence number is u8 according to docs
	internal_sequence: u8,
	pub write_buffer: [u8; MAX_FRAME_SIZE],
}

impl MavlinkServiceTx {
	fn new(io_service: RFD900Tx) -> MavlinkServiceTx {
		MavlinkServiceTx {
			io_service: io_service,
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
		// self.increment_internal_sequence();
		return current_sequence;
	}

	pub async fn write_raw(
		&mut self,
		buf: &[u8],
	) {
		self.io_service.write(buf).await;
	}
}

pub struct MavlinkServiceRx {
	pub io_service: RFD900Rx,
}
impl MavlinkServiceRx {
	fn new(io_service: RFD900Rx) -> MavlinkServiceRx {
		MavlinkServiceRx { io_service: io_service }
	}

	pub async fn read_frame(&mut self) -> Result<MAVLinkV2MessageRaw, MavlinkError> {
		match read_v2_raw_message_async::<MavMessage>(&mut self.io_service).await {
			Ok(frame) => Ok(frame),
			Err(_) => Err(MavlinkError::ReceiveFailed),
		}
	}

	pub async fn read_v1_frame(&mut self) -> Result<MAVLinkV1MessageRaw, MavlinkError> {
		match read_v1_raw_message_async::<MavMessage>(&mut self.io_service).await {
			Ok(frame) => Ok(frame),
			Err(_) => Err(MavlinkError::ReceiveFailed),
		}
	}

	pub async fn read_raw(
		&mut self,
		buf: &mut [u8],
	) {
		self.io_service.read(buf).await;
	}
}
