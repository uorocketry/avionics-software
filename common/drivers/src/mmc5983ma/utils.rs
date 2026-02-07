pub enum MMC5983MARegisters {
	XOut0 = 0x00,
	XOut1 = 0x01,
	YOut0 = 0x02,
	YOut1 = 0x03,
	ZOut0 = 0x04,
	ZOut1 = 0x05,
	XYZOut2 = 0x06,
	TOut = 0x07,
	Status = 0x08,
	InternalControl0 = 0x09,
	InternalControl1 = 0x0A,
	InternalControl2 = 0x0B,
	InternalControl3 = 0x0C,
	ProductID1 = 0x2F,
}

// Note that the increasing the bandwidth decreases time the internal filter is running for, resulting in a lower resolution measurement
#[derive(Clone)]
pub enum MMC5983MABandwidth {
	Hz100 = 0b00,
	Hz200 = 0b01,
	Hz400 = 0b10,
	Hz800 = 0b11,
}
