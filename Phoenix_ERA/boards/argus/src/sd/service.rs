// SHOULD DO: use embedded_hal traits instead of embassy_stm32 types directly

use core::str::FromStr;

use defmt::{error, trace};
use embassy_stm32::spi::{MisoPin, MosiPin, SckPin};
use embassy_stm32::{gpio, spi, time, Peripheral};
use embassy_time::Delay;
use embedded_sdmmc::{Error, Mode, VolumeIdx};
use heapless::{format, String, Vec};
use static_cell::StaticCell;

use crate::sd::config::{MAX_DIRS, MAX_FILES};
use crate::sd::types::{
	FakeTimeSource, FileName, Line, OperationScope, SDCardChipSelect, SDCardDirectory, SDCardInstance, SDCardSpiBus, SDCardSpiDevice,
	SDCardSpiRefCell, SDCardVolumeManager, SdCardError, SdCardWriteQueue,
};

// Hack: During SDCardService initialization, SpiMutex needs to be passed by reference to SpiDevice and they both need to be encapsulated within SDCardService
// Which is not possible because rust does not allow self-referencing structs so it's being made static cell instead of maintained inside SDCardService which is a singleton anyways
// Cannot be a pure static because it depends on peripherals which become available later
static SD_CARD_SPI_REFCELL: StaticCell<SDCardSpiRefCell> = StaticCell::new();

// Channel for queueing write operations
pub static SD_CARD_WRITE_QUEUE: SdCardWriteQueue = SdCardWriteQueue::new();

pub struct SDCardService {
	volume_manager: SDCardVolumeManager<MAX_DIRS, MAX_FILES>,
	pub current_session: Option<String<3>>, // Wrapped around option to so if None session has not been created yet
}

impl SDCardService {
	pub fn new<T: spi::Instance>(
		peri: impl Peripheral<P = T> + 'static,
		sck: impl Peripheral<P = impl SckPin<T>> + 'static,
		mosi: impl Peripheral<P = impl MosiPin<T>> + 'static,
		miso: impl Peripheral<P = impl MisoPin<T>> + 'static,
		cs: impl Peripheral<P = impl gpio::Pin> + 'static,
	) -> Self {
		// Only the SD Card is on this SPI Bus so the driver, mutex and, ref cell can be stored here and not shared
		let mut spi_config = spi::Config::default();
		spi_config.frequency = time::mhz(16);
		spi_config.bit_order = spi::BitOrder::MsbFirst;

		let mut spi_bus = SDCardSpiBus::new_blocking(peri, sck, mosi, miso, spi_config);

		// Idk why we do this @Noah Sprenger did it I'm copying this behavior blindly xD
		let data: [u8; 10] = [0xFF; 10];
		spi_bus.blocking_write(&data).unwrap();

		let spi_cs = SDCardChipSelect::new(cs, gpio::Level::High, gpio::Speed::Low);
		let spi_ref_cell: &'static mut SDCardSpiRefCell = SD_CARD_SPI_REFCELL.init(SDCardSpiRefCell::new(spi_bus));
		let spi_device: SDCardSpiDevice = SDCardSpiDevice::new(spi_ref_cell, spi_cs, Delay).unwrap();

		// Embedded SDMMC library setup
		let sd_card = SDCardInstance::new(spi_device, Delay);
		let volume_manager: SDCardVolumeManager<MAX_DIRS, MAX_FILES> = SDCardVolumeManager::new_with_limits(sd_card, FakeTimeSource::new(), 0);

		SDCardService {
			volume_manager,
			current_session: None,
		}
	}

	// Closure that handles accessing root directory
	pub fn with_root<T, E>(
		&mut self,
		f: impl for<'b> FnOnce(SDCardDirectory<'b, MAX_DIRS, MAX_FILES>) -> Result<T, SdCardError>,
	) -> Result<T, SdCardError> {
		trace!("Opening root directory");
		let volume = self.volume_manager.open_volume(VolumeIdx(0))?;
		let root_dir = volume.open_root_dir()?;
		f(root_dir)
	}

	// Non-blocking write that queues the message to be written by the async task
	pub async fn enqueue_write(
		scope: OperationScope,
		path: FileName,
		line: Line,
	) {
		trace!("Enqueuing write to SD card: {:?}, {:?}, {:?}", scope, path.as_str(), line.as_str());
		SD_CARD_WRITE_QUEUE.send((scope, path, line)).await;
	}

	pub fn delete(
		&mut self,
		scope: OperationScope,
		path: FileName,
	) -> Result<(), SdCardError> {
		trace!("Deleting from SD card: {:?}, {:?}", scope, path.as_str());

		// Setup all variables needed from self since we cannot access self inside the self.with_root closure
		let session = match scope {
			OperationScope::CurrentSession => Some(self.current_session.as_ref().unwrap().clone()),
			_ => None,
		};

		self.with_root::<(), SdCardError>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?,
			};

			let result = directory.delete_file_in_dir(path.as_str());
			match result {
				Ok(()) => {}               // Cool
				Err(Error::NotFound) => {} // If file not found, consider it deleted
				Err(e) => return Err(e),   // Propagate other errors
			}
			Ok(())
		})
	}

	// Blocking write that immediately writes the message to the SD card
	pub fn write(
		&mut self,
		scope: OperationScope,
		path: FileName,
		mut line: Line,
	) -> Result<(), SdCardError> {
		trace!("Writing to SD card: {:?}, {:?}, {:?}", scope, path.as_str(), line.as_str());

		// Ensure line ends with newline
		if !line.as_str().ends_with("\n") {
			let _ = line.push('\n'); // Ignore capacity error
		}

		let session = match scope {
			OperationScope::CurrentSession => match &self.current_session {
				Some(session) => Some(session.clone()),
				None => {
					error!("Current session is not set for writing to current session scope");
					Some(String::<3>::from_str("0").unwrap())
				}
			},
			_ => None,
		};

		self.with_root::<(), SdCardError>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?,
			};

			let file = directory.open_file_in_dir(path.as_str(), Mode::ReadWriteCreateOrAppend)?;
			file.write(line.as_bytes())?;
			file.flush()?;
			Ok(())
		})
	}

	pub fn read_fixed_number_of_lines<const LINES_COUNT: usize>(
		&mut self,
		scope: OperationScope,
		path: FileName,
	) -> Result<Vec<Line, LINES_COUNT>, SdCardError> {
		let mut lines: Vec<Line, LINES_COUNT> = Vec::new();
		self.read(scope, path, |line| {
			if lines.len() < LINES_COUNT {
				lines.push(line.clone()).ok(); // Ignore capacity error
				true
			} else {
				false
			}
		})?;
		Ok(lines)
	}

	pub fn file_exists(
		&mut self,
		scope: OperationScope,
		path: FileName,
	) -> Result<bool, SdCardError> {
		let session = match scope {
			OperationScope::CurrentSession => Some(self.current_session.as_ref().unwrap().clone()),
			_ => None,
		};

		self.with_root::<bool, SdCardError>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?,
			};
			let result = directory.open_file_in_dir(path.as_str(), Mode::ReadOnly);
			match result {
				Ok(_) => Ok(true),                 // File exists
				Err(Error::NotFound) => Ok(false), // Does not exist
				Err(e) => Err(e),                  // Propagate other errors
			}
		})
	}

	pub fn read<F: (FnMut(&Line) -> bool)>(
		&mut self,
		scope: OperationScope,
		path: FileName,
		mut handle_line: F,
	) -> Result<(), SdCardError> {
		// Setup all variables needed from self since we cannot access self inside the self.with_root closure

		trace!("Reading from SD card: {:?}, {:?}", scope, path.as_str());

		let session = match scope {
			OperationScope::CurrentSession => Some(self.current_session.as_ref().unwrap().clone()),
			_ => None,
		};

		self.with_root::<(), SdCardError>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?,
			};

			let file = directory.open_file_in_dir(path.as_str(), Mode::ReadOnly)?;

			// underlying read buffer (can be any small chunk)
			let mut read_buffer = [0u8; 256];

			// logical line buffer, bounded by `Line` capacity
			let mut line = Line::new();

			loop {
				let read_bytes_count = file.read(&mut read_buffer)?;
				if read_bytes_count == 0 {
					// EOF — emit any trailing partial line
					if !line.is_empty() {
						handle_line(&line);
						line.clear();
					}
					break;
				}

				for &read_byte in &read_buffer[..read_bytes_count] {
					match read_byte {
						b'\n' => {
							// End of line (LF). Emit and clear.
							if !handle_line(&line) {
								return Ok(()); // Stop reading if handler returns false
							}
							line.clear();
						}
						b'\r' => {
							// Ignore CR (handles CRLF). Do nothing.
						}
						_ => {
							// Push char if capacity allows; if full, emit as a line-chunk and continue
							if line.push(read_byte as char).is_err() {
								// Buffer full — emit current chunk as a line
								if !handle_line(&line) {
									return Ok(()); // Stop reading if handler returns false
								}
								line.clear();
								// Try pushing the current char again. It will fit on empty.
								let _ = line.push(read_byte as char);
							}
						}
					}
				}
			}

			Ok(())
		})
	}

	pub fn refresh_session(
		&mut self,
		session: i32,
	) -> Result<(), SdCardError> {
		trace!("Refreshing SD card service session to {}", session);

		// Create session directory if it doesn't exist
		let session_dir_name: String<3> = format!("{}", session).unwrap();
		self.with_root::<(), SdCardError>(|root_dir| {
			match root_dir.open_dir(session_dir_name.as_str()) {
				Ok(_) => Ok(()), // Directory exists
				Err(Error::NotFound) => {
					// Create directory
					root_dir.make_dir_in_dir(session_dir_name.as_str())?;
					Ok(())
				}
				Err(e) => Err(e), // Propagate other errors
			}
		})?;

		self.current_session = Some(session_dir_name);
		Ok(())
	}
}
