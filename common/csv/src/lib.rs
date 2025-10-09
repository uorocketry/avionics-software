#![no_std]
#![no_main]

use core::str::FromStr;

use heapless::String;
use serde::{Deserialize, Serialize};
use serde_csv_core::{Reader, Writer};

pub trait SerializeCSV<const MAX_LINE_SIZE: usize>: Serialize + for<'d> Deserialize<'d> {
	fn get_csv_header() -> String<MAX_LINE_SIZE>;
	fn from_csv_line(line: &String<MAX_LINE_SIZE>) -> Self {
		let mut reader = Reader::<255>::new();
		let (record, _n) = reader.deserialize::<Self>(line.as_bytes()).unwrap();
		record
	}
	fn to_csv_line(&self) -> String<MAX_LINE_SIZE> {
		let mut writer = Writer::new();
		let mut line = [0u8; 255];
		writer.serialize(&self, &mut line).unwrap();
		let line_str = core::str::from_utf8(&line).unwrap();

		String::from_str(line_str).unwrap()
	}
}
