use core::str::FromStr;

use defmt::info;
use heapless::{format, Vec};

use crate::sd::service::SDCardService;
use crate::sd::types::{FileName, Line, OperationScope, SdCardError};
use crate::utils::types::AsyncMutex;

/// Handles session management for data logging.
/// It reads the last session number from a session.txt file on the SD card, increments it,
/// and writes it back for the next boot. It also keeps track of the current session in memory.
pub struct SessionService {
	pub current_session: Option<i32>,
	sd_card_service: &'static AsyncMutex<SDCardService>,
	session_file_path: FileName,
}

impl SessionService {
	pub fn new(sd_card_service: &'static AsyncMutex<SDCardService>) -> Self {
		Self {
			current_session: None,
			sd_card_service,
			session_file_path: FileName::from_str("session.txt").unwrap(),
		}
	}

	pub async fn ensure_session(&mut self) -> Result<(), SdCardError> {
		if self.current_session.is_none() {
			self.refresh_session().await?;
		}
		Ok(())
	}

	pub async fn refresh_session(&mut self) -> Result<(), SdCardError> {
		let mut sd_service = self.sd_card_service.lock().await;
		let mut current_session = 0;

		// If session file exists, read the previous session number, increment it, and write it back
		if sd_service.file_exists(OperationScope::Root, self.session_file_path.clone())? {
			let lines: Vec<Line, 1> = sd_service.read_fixed_number_of_lines::<1>(OperationScope::Root, self.session_file_path.clone())?;
			let previous_session = lines[0].as_str().parse::<i32>().unwrap_or(0);
			current_session = previous_session + 1;

			sd_service.delete(OperationScope::Root, self.session_file_path.clone())?;
		}

		info!("Created a new session: {}", current_session);

		// Write the new session number to the session file so the next boot can read it and increment
		sd_service.write(
			OperationScope::Root,
			self.session_file_path.clone(),
			format!("{}", current_session).unwrap(),
		)?;

		// Update the current session in both services
		self.current_session = Some(current_session);
		sd_service.refresh_session(current_session)?;
		Ok(())
	}
}
