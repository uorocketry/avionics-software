use embassy_time::Instant;
use mavlink::{MAVLinkV2MessageRaw, MavHeader};
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;

pub struct PressureSensors {
	pub ps_1: u32,
	pub ps_2: u32,
	pub ps_3: u32,
	pub ps_4: u32,
	pub ps_5: u32,
	pub ps_6: u32,
	pub ps_7: u32,
	pub ps_8: u32,
}

pub struct PressureDataPublisher<const N: usize> {
	pressure_sensor_data: [PressureSensors; N],
	timestamp: u32,
	page_num: u8,
	page_total: u8,
	publish_done: bool,
}

impl<const N: usize> PressureDataPublisher<N> {
	pub fn new(pressure_sensor_data: [PressureSensors; N]) -> PressureDataPublisher<N> {
		PressureDataPublisher {
			pressure_sensor_data: pressure_sensor_data,
			timestamp: 0,
			page_num: 1,
			page_total: N as u8,
			publish_done: false,
		}
	}

	pub fn update(
		&mut self,
		pressure_sensor_data: [PressureSensors; N],
	) {
		self.pressure_sensor_data = pressure_sensor_data;
	}
}

// impl<const N: usize> Publisher for PressureDataPublisher<N> {
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

// 		let pressure_page = &self.pressure_sensor_data[(self.page_num - 1) as usize];

// 		let payload = mavlink::uorocketry::PRESSURE_UOR_DATA {
// 			PS_1: pressure_page.ps_1,
// 			PS_2: pressure_page.ps_2,
// 			PS_3: pressure_page.ps_3,
// 			PS_4: pressure_page.ps_4,
// 			PS_5: pressure_page.ps_5,
// 			PS_6: pressure_page.ps_6,
// 			PS_7: pressure_page.ps_7,
// 			PS_8: pressure_page.ps_8,
// 			PAGE_TOTAL: self.page_total,
// 			PAGE_NUM: self.page_num,
// 			time_boot_ms: self.timestamp,
// 		};
// 		frame.serialize_message_data(header, &payload);

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
