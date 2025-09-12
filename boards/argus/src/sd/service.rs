use defmt::{debug, error, Debug2Format};
use embassy_stm32::spi::{MisoPin, MosiPin, SckPin};
use embassy_stm32::{gpio, spi, time, Peripheral};
use embassy_time::Delay;

use embedded_sdmmc::{Error, Mode, SdCardError, VolumeIdx};

use heapless::String;
use static_cell::StaticCell;

use crate::sd::time_source::FakeTimeSource;
use crate::sd::types::{FilePath, Line, OperationScope, SDCardChannel, SDCardChipSelect, SDCardDirectory, SDCardInstance, SDCardSpiBus, SDCardSpiDevice, SDCardSpiRefCell, SDCardVolumeManager};
use crate::utils::types::AsyncMutex;

// Maximum number of session directories allowed
const MAX_SESSIONS_COUNT: usize = 4;

// Max number of files before embedded-sdmmc overflows
const MAX_FILES_COUNT: usize = 4;

// Max number of messages allowed in the channel before it locks up until the channel clears
const QUEUE_SIZE: usize = 8;

// Hack: During SDCardService initialization, SpiMutex needs to be passed by reference to SpiDevice and they both need to be encapsulated within SDCardService
// Which is not possible because rust does not allow self-referencing structs so it's being made static cell instead of maintained inside SDCardService which is a singleton anyways
// Cannot be a pure static because it depends on peripherals which become available later
static SD_CARD_SPI_REFCELL: StaticCell<SDCardSpiRefCell> = StaticCell::new();

// Channel to allow deferring the action of writing to the sd card
#[allow(dead_code)]
static SD_CARD_CHANNEL: SDCardChannel<QUEUE_SIZE> = SDCardChannel::new();

pub struct SDCardService{
	volume_manager: SDCardVolumeManager<MAX_SESSIONS_COUNT, MAX_FILES_COUNT>,
	current_session: Option<String<3>>, // Wrapped around option to so if None session has not been created yet
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
		let volume_manager: SDCardVolumeManager<MAX_SESSIONS_COUNT, MAX_FILES_COUNT> = SDCardVolumeManager::new_with_limits(sd_card, FakeTimeSource::new(), 0);

		return SDCardService {
			volume_manager,
			current_session: None,
		};
	}

	// Closure that handles accessing root directory
	pub fn with_root<T, E>(
		&mut self,
		f: impl for<'b> FnOnce(
			SDCardDirectory<'b, MAX_SESSIONS_COUNT, MAX_FILES_COUNT>
		) -> Result<T, Error<SdCardError>>,
	) -> Result<T, Error<SdCardError>> {
		debug!("Opening root directory");
		let volume = self.volume_manager.open_volume(VolumeIdx(0))?;
		let root_dir = volume.open_root_dir()?;
		return f(root_dir);
	}

	/**
	 * Embassy task can call this to setup the sd card deferred write task
	 */
	pub async fn ensure_task(service_mutex: &'static AsyncMutex<Self>) {
		if let Err(error) = service_mutex.lock().await.ensure_session_created() {
			error!("Could not create session directory: {:?}", Debug2Format(&error));
		}

		debug!("Starting SD card write loop.");
		loop {
			let (scope, path, line) = SD_CARD_CHANNEL.receiver().receive().await;
			if service_mutex.lock().await.write(scope, path, line).is_err() {
				error!("Could not write to SD card.");
				continue;
			}
		}
	}

	// Non-blocking write that queues the message to be written by the async task
	pub async fn enqueue_write(
		scope: OperationScope,
		path: FilePath,
		line: Line
	) {
		debug!("Enqueuing write to SD card: {:?}, {:?}, {:?}", scope, path.as_str(), line.as_str());
		SD_CARD_CHANNEL.send((scope, path, line)).await;
	}

	pub fn delete(
		&mut self,
		scope: OperationScope,
		path: FilePath,
	) {
		debug!("Deleting from SD card: {:?}, {:?}", scope, path.as_str());

		// Setup all variables needed from self since we cannot access self inside the self.with_root closure

		let session = match scope {
			OperationScope::CurrentSession => {
				Some(self.current_session.as_ref().unwrap().clone())
			},
			_ => None
		};

		self.with_root::<(), Error<SdCardError>>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?
			};

			directory.delete_file_in_dir(path.as_str())?;
			Ok(())
		}).unwrap();
	}

	// Blocking write that immediately writes the message to the SD card
	pub fn write(
		&mut self,
		scope: OperationScope,
		path: FilePath,
		line: Line
	) -> Result<(), Error<SdCardError>> {
		debug!("Writing to SD card: {:?}, {:?}, {:?}", scope, path.as_str(), line.as_str());

		// Ensure session directory is created if writing to current session
		let session = match scope {
			OperationScope::CurrentSession => {
				self.ensure_session_created()?;
				Some(self.current_session.as_ref().unwrap().clone())
			},
			_ => None
		};

		self.with_root::<(), Error<SdCardError>>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?
			};

			let file = directory.open_file_in_dir(
				path.as_str(),
				Mode::ReadWriteCreateOrAppend
			)?;
			file.write(line.as_bytes())?;
			file.flush()?;
			Ok(())
		})
	}

	pub fn read<F: FnMut(&Line)>(
		&mut self,
		scope: OperationScope,
		path: FilePath,
		mut handle_line: F
	) -> Result<(), Error<SdCardError>> {
		// Setup all variables needed from self since we cannot access self inside the self.with_root closure

		let session = match scope {
			OperationScope::CurrentSession => {
				Some(self.current_session.as_ref().unwrap().clone())
			},
			_ => None
		};

		self.with_root::<(), Error<SdCardError>>(|root_dir| {
			let directory = match scope {
				OperationScope::Root => root_dir,
				OperationScope::CurrentSession => root_dir.open_dir(session.unwrap().as_str())?
			};

			let file = directory.open_file_in_dir(
				path.as_str(),
				Mode::ReadOnly
			)?;

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
							handle_line(&line);
							line.clear();
						}
						b'\r' => {
							// Ignore CR (handles CRLF). Do nothing.
						}
						_ => {
							// Push char if capacity allows; if full, emit as a line-chunk and continue
							if line.push(read_byte as char).is_err() {
								// Buffer full — emit current chunk as a line
								handle_line(&line);
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

	pub fn ensure_session_created(&mut self) -> Result<(), Error<SdCardError>> {
		debug!("Ensuring session directory is created");

		if self.current_session.is_some() {
			// Session directory already created
			return Ok(());
		}

		let last_session_number = self.get_last_session_number()?;
		let current_session = last_session_number + 1;

		// Cast to str
		let mut current_session_buffer = itoa::Buffer::new();
		let current_session_str = current_session_buffer.format(current_session);
		self.current_session = Some(String::new());
		self.current_session.as_mut().unwrap().push_str(current_session_str).ok(); // Ignore capacity error 

		self.with_root::<(), Error<SdCardError>>(|root_dir| {
			debug!("Creating session directory: {}", current_session_str);
			return root_dir.make_dir_in_dir(current_session_str);
		})
	}

	/**
	 * Infer the last session based on the largest directory in the SD Card 
	 */
	fn get_last_session_number(&mut self) -> Result<u8, Error<SdCardError>> {
		debug!("Getting last session number");
		// Sessions are directories generated on root directory: numbers starting from 0 autoincrementing
		let mut last_session: u8 = 0;

		return self.with_root::<u8, Error<SdCardError>>(|root_dir| {
			root_dir.iterate_dir(|entry| {
				if !entry.attributes.is_directory() {
					return;
				}

				let name = get_name_from_basename(entry.name.base_name());
				let current_session = name.parse::<u8>().unwrap_or(0);
				if current_session > last_session {
					last_session = current_session;
				}
			})?;

			debug!("Last session number: {}", last_session);
			return Ok(last_session);
		});
	}
}

/**
 * Get the name of a file or directory from its basename i.e. remove the extension
 * Example: foo.txt -> foo
 */
fn get_name_from_basename<'b>(bytes: &'b[u8]) -> &'b str {
	let mut end = bytes.len();
	while end > 0 && bytes[end - 1] == b' ' {
		end -= 1;
	}
	return core::str::from_utf8(&bytes[..end]).unwrap()
}