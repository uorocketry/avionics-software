// THIS FILE IS ONLY USED FOR TESTING WITHOUT HARDWARE!

use embedded_io_async::{Error, ErrorType, Read, Write};

use crate::data_structs::messages::SbgEcomLogMag;
use crate::sbg_device::SbgDevice;

// Only exists to satify the requirements for a struct who implements the "Read" trait
#[derive(Debug, Copy, Clone)]
pub enum ErrorSbg {
	TestingError(),
}

// "Error" trait wants the error enum to implement the "Display" trait
impl core::fmt::Display for ErrorSbg {
	fn fmt(
		&self,
		f: &mut core::fmt::Formatter<'_>,
	) -> core::fmt::Result {
		let message = match self {
			// It should have not reached this state as there is no reason for an error to ever be raised! (yet)
			Self::TestingError() => "It should have never reached this state",
		};

		write!(f, "{}", message)
	}
}

impl core::error::Error for ErrorSbg {}

// Implements the error trait for the enum
impl embedded_io_async::Error for ErrorSbg {
	fn kind(&self) -> embedded_io_async::ErrorKind {
		embedded_io_async::ErrorKind::Unsupported
	}
}

// Emulates the microcontroller's UART line for algorithim logic.
pub struct SbgTester {
	// Two internal buffers to simulate a fragmented frame recieved from SBG. First buffer is first fragment and second buffer is second fragment
	internal_buffer_1: [u8; 4096],
	internal_buffer_2: [u8; 4096],
	write_buffer: [u8; 4096],
	pub read_count: u8,
}

impl SbgTester {
	// Populates internal buffers and returns a new struct
	pub fn new(
		internal_buffer_1: &[u8],
		internal_buffer_2: &[u8],
	) -> SbgTester {
		let mut intermediate_1: [u8; 4096] = [0; 4096];
		let mut intermediate_2: [u8; 4096] = [0; 4096];

		let mut index = 0;

		for i in internal_buffer_1 {
			// To test split frames, feed from end
			intermediate_1[(4096 - internal_buffer_1.len()) + index] = i.clone();
			index += 1;
		}

		index = 0;

		for i in internal_buffer_2 {
			intermediate_2[index] = i.clone();
			index += 1;
		}
		SbgTester {
			internal_buffer_1: intermediate_1,
			internal_buffer_2: intermediate_2,
			write_buffer: [0; 4096],
			read_count: 0,
		}
	}
}

impl ErrorType for SbgTester {
	type Error = ErrorSbg;
}

impl Read for SbgTester {
	// Reads to buffer passed as arguement. Switches between first and second fragment based on value of "read_count" field
	async fn read(
		&mut self,
		buf: &mut [u8],
	) -> Result<usize, ErrorSbg> {
		let mut index = 0;

		// When the tester has not been read from 	yet or read from an even amount of times
		if self.read_count % 2 == 0 {
			for x in self.internal_buffer_1 {
				buf[index] = x.clone();
				index += 1;
			}
		}
		// When the tester has been read from an odd amount of times
		else {
			for x in self.internal_buffer_2 {
				buf[index] = x.clone();
				index += 1;
			}
		}
		self.read_count += 1;
		return Result::Ok(index);
	}
}

// Not properly implemented yet
impl Write for SbgTester {
	async fn write(
		&mut self,
		buf: &[u8],
	) -> Result<usize, Self::Error> {
		let mut index = 0;
		for i in buf {
			self.write_buffer[index] = i.clone();
			index += 1;
		}
		panic!("{:?}", self.write_buffer);
		return Result::Ok(index);
	}

	async fn flush(&mut self) -> Result<(), Self::Error> {
		todo!()
	}
}
