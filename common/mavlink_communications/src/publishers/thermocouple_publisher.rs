use embassy_time::Instant;
use mavlink::{MAVLinkV2MessageRaw, MavHeader};
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;

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

// impl<const N: usize> Publisher for ThermocoupleDataPublisher<N> {
// 	fn publish(
// 		self: &mut Self,
// 		sequence: u8,
// 		buff: &mut [u8],
// 	) -> usize {
// 		self.timestamp = Instant::now().as_millis() as u32;
// 		let mut frame = MAVLinkV2MessageRaw::new();

// 		let header = MavHeader {
// 			system_id: 1,
// 			component_id: 1,
// 			sequence: sequence,
// 		};

// 		let thermocouple_page = &self.thermocouple_data[(self.page_num - 1) as usize];

// 		let payload = mavlink::uorocketry::THERMOCOUPLE_UOR_DATA {
// 			TC_1: thermocouple_page.tc_1,
// 			TC_2: thermocouple_page.tc_2,
// 			TC_3: thermocouple_page.tc_3,
// 			TC_4: thermocouple_page.tc_4,
// 			TC_5: thermocouple_page.tc_5,
// 			TC_6: thermocouple_page.tc_6,
// 			TC_7: thermocouple_page.tc_7,
// 			TC_8: thermocouple_page.tc_8,
// 			PAGE_TOTAL: self.page_total,
// 			PAGE_NUM: self.page_num,
// 			time_boot_ms: self.timestamp,
// 		};
// 		self.page_num += 1;
// 		frame.serialize_message_data(header, &payload);

// 		let frame_size = frame.raw_bytes().len();

// 		buff[0..frame_size].copy_from_slice(frame.raw_bytes());

// 		if self.page_num == self.page_total {
// 			self.page_num = 1;
// 			self.publish_done = true;
// 			return frame_size;
// 		}
// 		return frame_size;
// 	}
// }
