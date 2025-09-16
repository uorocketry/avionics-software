use core::str::FromStr;

use heapless::String;
use serde::Serialize;
use serde_csv_core::Writer;

use crate::sd::types::Line;

pub trait SerializeCSV: Serialize {
	fn get_header() -> Line;
	fn to_csv_line(
		&self,
		writer: &mut Writer,
	) -> Line {
		let mut line = [0; 255];
		writer.serialize(&self, &mut line).unwrap();
		let line_str = core::str::from_utf8(&line).unwrap();
		let line_string = String::from_str(line_str).unwrap();
		line_string
	}
}
