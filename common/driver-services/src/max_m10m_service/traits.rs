use ublox::{packetref_proto31::PacketOwned, proto31::PacketRef};

pub trait MaxM10MListener {
	fn __update(
		&mut self,
		packet: &PacketRef,
	);
}
