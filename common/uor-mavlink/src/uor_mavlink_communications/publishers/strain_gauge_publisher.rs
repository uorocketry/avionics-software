use embassy_time::Instant;
use uor_mavlink_macros::Publisher;

use crate::uor_mavlink_communications_traits::publisher::Publisher;
use crate::uor_mavlink_dialect::{MAVLinkV2MessageRaw, MavHeader, common::DEBUG_DATA};

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
