use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use crate::sd::config::QUEUE_SIZE;
use crate::sd::types::{files::OperationScope, FileName, Line};

pub type SdCardWriteQueue = Channel<CriticalSectionRawMutex, (OperationScope, FileName, Line), QUEUE_SIZE>;
