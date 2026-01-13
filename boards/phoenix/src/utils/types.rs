use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

/// Shorthand for an async mutex safe to share between executors and interrupts.
pub type AsyncMutex<T> = Mutex<CriticalSectionRawMutex, T>;
