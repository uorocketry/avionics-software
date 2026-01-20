use ublox::{nav_pos_llh::NavPosLlhOwned, packetref_proto31::PacketOwned, proto31::PacketRef};

use crate::max_m10m_service::traits::MaxM10MListener;

pub struct NavPosLlhListener {
	pub internal: Option<NavPosLlhOwned>,
	pub new_data: bool,
}

impl NavPosLlhListener {
	pub fn new() -> Self {
		NavPosLlhListener {
			internal: None,
			new_data: false,
		}
	}
}

impl MaxM10MListener for NavPosLlhListener {
	fn __update(
		&mut self,
		packet: &ublox::packetref_proto31::PacketRef,
	) {
		if let PacketRef::NavPosLlh(payload) = packet {
			self.internal = Some(payload.to_owned());
			self.new_data = true
		}
	}
}
