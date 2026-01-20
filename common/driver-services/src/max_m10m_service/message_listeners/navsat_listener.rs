use ublox::{
	nav_pos_llh::NavPosLlhOwned,
	nav_sat::{NavSat, NavSatOwned},
	packetref_proto31::PacketOwned,
	proto31::PacketRef,
};

use crate::max_m10m_service::traits::MaxM10MListener;

pub struct NavSatListener {
	pub internal: Option<NavSatOwned>,
	pub new_data: bool,
}

impl NavSatListener {
	pub fn new() -> Self {
		NavSatListener {
			internal: None,
			new_data: false,
		}
	}
}

impl MaxM10MListener for NavSatListener {
	fn __update(
		&mut self,
		packet: &ublox::packetref_proto31::PacketRef,
	) {
		if let PacketRef::NavSat(payload) = packet {
			self.internal = Some(payload.to_owned());
			self.new_data = true
		}
	}
}
