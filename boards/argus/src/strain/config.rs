// Size of the queue used to send strain readings from the strain service to the SD card service
pub const STRAIN_READING_QUEUE_SIZE: usize = 16;

// File name used to read/write linear transformations that applied to strain readings to/from the SD card
// Linear transformations are stored in CSV format
pub const LINEAR_TRANSFORMATIONS_FILE_NAME: &str = "t_strain.csv"; // Cannot be longer than 12 characters
