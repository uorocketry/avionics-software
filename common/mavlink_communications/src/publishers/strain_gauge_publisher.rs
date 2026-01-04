use embassy_time::Instant;
use mavlink::{MAVLinkV2MessageRaw, MavHeader};
use mavlink_communications_traits::publish_subscribe_tools::publisher::Publisher;
// TODO: Implement revised multi=page publisher design
pub struct StrainGauges {
	pub sg_1: u32,
	pub sg_2: u32,
	pub sg_3: u32,
	pub sg_4: u32,
	pub sg_5: u32,
	pub sg_6: u32,
	pub sg_7: u32,
	pub sg_8: u32,
}

pub struct StrainDataPublisher {
	strain_gauge_data: StrainGauges,
	page_num: u8,
	page_total: u8,
	timestamp: u32,
}

impl StrainDataPublisher {
	pub fn new(
		strain_gauge_data: StrainGauges,
		page_num: u8,
		page_total: u8,
	) -> StrainDataPublisher {
		StrainDataPublisher {
			strain_gauge_data: strain_gauge_data,
			page_num: page_num,
			page_total: page_total,
			timestamp: 0,
		}
	}

	pub fn update(
		&mut self,
		strain_gauge_data: StrainGauges,
	) {
		self.strain_gauge_data = strain_gauge_data;
	}
}

// impl Publisher for StrainDataPublisher {
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

// 		let payload = mavlink::uorocketry::STRAIN_UOR_DATA {
// 			SG_1: self.strain_gauge_data.sg_1,
// 			SG_2: self.strain_gauge_data.sg_2,
// 			SG_3: self.strain_gauge_data.sg_3,
// 			SG_4: self.strain_gauge_data.sg_4,
// 			SG_5: self.strain_gauge_data.sg_5,
// 			SG_6: self.strain_gauge_data.sg_6,
// 			SG_7: self.strain_gauge_data.sg_7,
// 			SG_8: self.strain_gauge_data.sg_8,
// 			PAGE_TOTAL: self.page_total,
// 			PAGE_NUM: self.page_num,
// 			time_boot_ms: self.timestamp,
// 		};

// 		frame.serialize_message_data(header, &payload);

// 		let frame_size = frame.raw_bytes().len();

// 		buff[0..frame_size].copy_from_slice(frame.raw_bytes());
// 		return frame_size;
// 	}
// }
