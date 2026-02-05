use embassy_time::Instant;
use uor_mavlink_macros::Publisher;

use crate::uor_mavlink_communications_traits::publisher::Publisher;
use crate::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::DEBUG_DATA};

pub struct Thermocouples {
	pub tc_1: u32,
	pub tc_2: u32,
	pub tc_3: u32,
	pub tc_4: u32,
	pub tc_5: u32,
	pub tc_6: u32,
	pub tc_7: u32,
	pub tc_8: u32,
}

pub struct ThermocoupleDataPublisher<const N: usize> {
	thermocouple_data: [Thermocouples; N],
	timestamp: u32,
	page_num: u8,
	page_total: u8,
	publish_done: bool,
}

impl<const N: usize> ThermocoupleDataPublisher<N> {
	pub fn new(thermocouple_data: [Thermocouples; N]) -> ThermocoupleDataPublisher<N> {
		ThermocoupleDataPublisher {
			thermocouple_data: thermocouple_data,
			timestamp: 0,
			page_num: 1,
			page_total: N as u8,
			publish_done: false,
		}
	}

	pub fn publish_done(&mut self) -> bool {
		let result = self.publish_done;
		self.publish_done = false;
		return result;
	}

	pub fn update(
		&mut self,
		thermocouple_data: [Thermocouples; N],
	) {
		self.thermocouple_data = thermocouple_data;
	}
}
