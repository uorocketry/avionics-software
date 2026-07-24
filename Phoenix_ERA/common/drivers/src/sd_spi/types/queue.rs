use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::sd_spi::config::SD_WRITING_QUEUE_SIZE;
use crate::sd_spi::types::{FileName, Line, files::OperationScope};

pub type SdCardWriteQueue = Channel<CriticalSectionRawMutex, (OperationScope, FileName, Line), SD_WRITING_QUEUE_SIZE>;
