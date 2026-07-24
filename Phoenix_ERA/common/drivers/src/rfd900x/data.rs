#[derive(Clone)]
// For Peer to Peer Firmware
pub enum Registers {
	/// Serial speed/baud-rate
	/// in ‘one-byte form’. Accepted values are 1, 2, 4, 9, 19, 38, 57, 115, 230, 460 and 1000
	/// corresponding to 1200bps, 2400bps, 4800bps, 9600bps, 19200bps, 38400bps,
	/// 57600bps, 115200bps, 230400bps, 460800bps and 1000000bps respectively.
	SerialSpeed = 1,
	/// Air data rate (like baud rate, but for the antenna communication)
	/// Accepted values are 12, 56, 64, 100, 125, 200, 224, 500 and 750
	/// corresponding to 12000bps, 56000bps 64000bps, 100000bps, 125000bps,
	/// 200000bps, 250000bps, 224000bps, 500000bps and 750000bps respectively.
	AirSpeed = 2,
	/// Network ID
	NetId = 3,
	/// Transmit power  (power in dBm)
	TxPower = 4,
	/// [Golay](https://en.wikipedia.org/wiki/Binary_Golay_code) error correct code enable/disable
	ECC = 5,
	/// Mavlink framing and reporting enable/disable
	Mavlink = 6,
	/// Enable/disable packet resend (if bandwith allows for it)
	OpResend = 7,
	/// Minimum frequency
	MinFreq = 8,
	/// Maximum frequency
	MaxFreq = 9,
	/// Number of frequency hopping channels
	NumChannels = 10,
	/// Percent of time for transmit
	DutyCycle = 11,
	/// Listen before talk threshold
	LBTRSSI = 12,
	/// Ready-to-send and clear-to-send enable/disable
	RTSCTS = 13,
	/// Max transmit window size
	MaxWindow = 14,
	/// Encryption level (0 = off, 1 = 128bit AES)
	EncryptionLevel = 15,
	/// Selecting what antennas are in use
	/// (0 = Diversity, 1 = Antenna 1 only, 2 = Antenna 2 only, 3 = Antenna 1 TX and antenna 2 RX )
	AntennaMode = 20,
}

#[derive(Copy, Clone, Debug, defmt::Format)]
pub enum EncryptionLevel {
	Off = 0,
	AES = 1,
}

pub enum AntennaMode {
	Diversity = 0,
	Antenna1Only = 1,
	Both = 2,
}
