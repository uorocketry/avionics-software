pub const RW_MASK: u16 = 0b1000_0000;

pub const X_MASK: u32 = 0b1100_0000;
pub const Y_MASK: u32 = 0b0011_0000;
pub const Z_MASK: u32 = 0b0000_1100;

pub const X_OFFSET: u32 = 6;
pub const Y_OFFSET: u32 = 4;
pub const Z_OFFSET: u32 = 2;

// An additional millisecond is added to ensure measurement is completed (data sheet values are values listed here -1)
pub const HZ100_DELAY: u64 = 9;
pub const HZ200_DELAY: u64 = 5;
pub const HZ400_DELAY: u64 = 3;
pub const HZ800_DELAY: u64 = 1;
