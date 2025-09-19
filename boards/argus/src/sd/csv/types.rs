use core::str::FromStr;

use argus::sd::types::Line;
use heapless::String;
use serde::Serialize;
use serde_csv_core::Writer;

pub trait SerializeCSV: Serialize {
	fn get_header() -> Line;
	fn to_csv_line(&self) -> Line {
		let mut writer = Writer::new();
		let mut line = [0; 255];
		writer.serialize(&self, &mut line).unwrap();
		let line_str = core::str::from_utf8(&line).unwrap();
		let line_string = String::from_str(line_str).unwrap();
		line_string
	}
}
