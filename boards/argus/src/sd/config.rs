// Max number of directories that can be kept open at the same time before embedded-sdmmc overflows
pub const MAX_DIRS: usize = 4;

// Max number of files that can be kept open before embedded-sdmmc overflows
pub const MAX_FILES: usize = 4;

// Max number of messages allowed in the sd operation queue channel before it locks up until the channel clears
pub const QUEUE_SIZE: usize = 8;

// Agreed upon value for maximum length of a line that can be written to the SD card
pub const MAX_LINE_LENGTH: usize = 255;
