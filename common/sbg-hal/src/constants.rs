pub const MESSAGE_FIELD_OFFSET: usize = 2;
pub const CLASS_FIELD_OFFSET: usize = 3;
pub const LENGTH_OFFSET_HIGH: usize = 5;
pub const LENGTH_OFFSET_LOW: usize = 4;

// End bit offset is based on end of data segment, not start of packet
pub const END_BIT_OFFSET: usize = 3;

pub const PRE_DATA_OFFSET_TRUE: usize = 6;
