use core::str::FromStr;

use heapless::String;
use serde::Serialize;
use serde_csv_core::Writer;

pub trait SerializeCSV<const T: usize = 255>: Serialize {
	fn get_header() -> String<T>;
	fn to_csv_line(
		&self,
		writer: &mut Writer,
	) -> String<T> {
		let mut line = [0; T];
		writer.serialize(&self, &mut line).unwrap();
		let line_str = core::str::from_utf8(&line).unwrap();
		let line_string = String::from_str(line_str).unwrap();
		line_string
	}
}
