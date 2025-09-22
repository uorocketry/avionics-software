// Number of thermocouple channels per ADC
// Note: Not to get confused with the number of analog input channels on each ADC
// Each thermocouple channel uses a pair of analog input channels (differential measurement)
pub const CHANNEL_COUNT: usize = 4;

// Size of the queue used to send temperature readings from the temperature service to the SD card service
pub const QUEUE_SIZE: usize = 16;

// Maximum number of calibration data points allowed to be collected during a calibration session per thermocouple channel
pub const MAX_CALIBRATION_DATA_POINTS: usize = 10;

// File name used to read/write linear transformations that applied to thermocouple readings to/from the SD card
// Linear transformations are stored in CSV format
pub const LINEAR_TRANSFORMATIONS_FILE_NAME: &str = "t.csv"; // Cannot be longer than 12 characters
