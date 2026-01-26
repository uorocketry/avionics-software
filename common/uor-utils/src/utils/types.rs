use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;

pub type AsyncMutex<T> = Mutex<CriticalSectionRawMutex, T>;
