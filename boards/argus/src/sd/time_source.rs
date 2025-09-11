use embedded_sdmmc::{TimeSource, Timestamp};
use core::marker::PhantomData;

pub struct SDCardTimeSource {
	_marker: PhantomData<*const ()>,
}

impl SDCardTimeSource {
	pub fn new() -> Self {
		SDCardTimeSource {
			_marker: PhantomData,
		}
	}
}

impl TimeSource for SDCardTimeSource {
	fn get_timestamp(&self) -> Timestamp {
		// SHOULD DO: replace with an actual time source like RTC
		Timestamp {
			year_since_1970: 55,	// 2025
			zero_indexed_month: 0,	// Jan
			zero_indexed_day: 0,	// 1st
			hours: 0,
			minutes: 0,
			seconds: 0,
		}
	}
}