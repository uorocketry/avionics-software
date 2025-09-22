use defmt::Format;
use embedded_sdmmc::{Directory, VolumeManager};
use heapless::String;

use crate::sd::config::MAX_LINE_LENGTH;
use crate::sd::types::spi::SDCardInstance;
use crate::sd::types::time_source::FakeTimeSource;

pub type SDCardVolumeManager<const MAX_DIRS: usize, const MAX_FILES: usize> = VolumeManager<
	SDCardInstance,
	FakeTimeSource,
	MAX_DIRS, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES,
	1,
>;

pub type SDCardDirectory<'a, const MAX_DIRS: usize, const MAX_FILES: usize> = Directory<
	'a,
	SDCardInstance,
	FakeTimeSource,
	MAX_DIRS, // MAX_DIRS translates to MaxSessions count because each session is a directory
	MAX_FILES,
	1,
>;

pub type FileName = String<12>; // FAT 8.3 format only allows 8 chars for name, 3 for extension and 1 for the dot
pub type DirectoryName = String<8>; // Max directory name length in FAT 8.3 is 8 characters
pub type Line = String<MAX_LINE_LENGTH>; // A line to be written to the SD card

// Represents the scope of a read/write operation
#[derive(Format)]
pub enum OperationScope {
	Root,           // Reads/Writes the file in the absolute path specified
	CurrentSession, // Reads/Writes the file in the current session directory
}
