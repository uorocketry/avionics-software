use mavlink_core::MessageData;

pub trait MavlinkMapped<D: MessageData> {
	fn convert_to_mav(&mut self) -> D;
	fn convert_from_mav(frame: D) -> Self;
}
