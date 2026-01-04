use mavlink::common::MavState;

/// ID of system (vehicle) sending the message. Used to differentiate systems on network. Can have up to 255 but reduced to 10 as it is more than enough
pub enum SystemId {
	Id1 = 1,
	Id2 = 2,
	Id3 = 3,
	Id4 = 4,
	Id5 = 5,
	Id6 = 6,
	Id7 = 7,
	Id8 = 8,
	Id9 = 9,
	Id10 = 10,
}
