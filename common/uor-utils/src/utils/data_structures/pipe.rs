use core::error::Error;

use crate::utils::types::AsyncMutex;
pub enum PipeError {
	WriterExists,
}

pub struct Pipe<T: Default + Clone> {
	internal: T,
	has_writer: bool,
}

impl<'a, T: Default + Clone> Pipe<T> {
	pub fn new() -> Pipe<T> {
		Pipe {
			internal: T::default(),
			has_writer: false,
		}
	}

	pub fn create_input(&'a mut self) -> Result<PipeIn<'a, T>, PipeError> {
		if self.has_writer {
			Err(PipeError::WriterExists)
		} else {
			Ok(PipeIn {
				internal: &mut self.internal,
			})
		}
	}

	pub fn read(&self) -> T {
		self.internal.clone()
	}
}

pub struct PipeIn<'a, T> {
	internal: &'a mut T,
}

impl<'a, T> PipeIn<'a, T> {
	pub fn write(
		&mut self,
		value: T,
	) {
		*self.internal = value;
	}
}
