use embassy_time::Instant;
use uor_proc_macros::Publisher;

use crate::uor_mavlink::uor_mavlink_communications_traits::publisher::Publisher;
use crate::uor_mavlink::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, uorocketry::PRESSURE_UOR_DATA};

// #[derive(Publisher)]
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
