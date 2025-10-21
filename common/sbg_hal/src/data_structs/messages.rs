
// #![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use bytemuck::{Pod, Zeroable}; 
//* PLEASE NOTE THAT THE STRUCT'S FIELDS ARE ORDER SENSITIVE, USE <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html> FOR REFERENCE */





// SBG_ECOM_CLASS_LOG_ECOM_0 -- Message 01 SBG_ECOM_LOG_STATUS -- <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#SBG_ECOM_LOG_STATUS>
/// Struct for the SBG_ECOM_LOG_STATUS message

#[repr(C, packed)]
    #[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct SbgEcomLogStatus {
    /// Time since sensor is powered up in µs
    pub time_stamp: u32,
    /// General device status, <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#STATUS_GENERAL_STATUS>
    pub general_status: u16,
    /// Additional communication status ** ONLY SUPPORTED IN PRODUCTS RUNNING FIRMWARE V4.0 OR ABOVE **, <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#STATUS_COM_STATUS_2>
    pub com_status_2: u16,
    /// Communication status, <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#STATUS_COM_STATUS_2>
    pub com_status: u32,
    /// Aiding equipment status, <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#STATUS_COM_STATUS>
    pub aiding_status: u32, 
    /// Reserved status field for future use
    pub reserved_2: u32,
    /// Reserved field for future use
    pub reserved_3: u16,
    /// System up time since the power on - 0 if N/A (seconds)
    pub up_time: u32,
    /// Main CPU usage in percent - 0xFF if N/A
    pub cpu_usage: u32,
}


// SBG_ECOM_CLASS_LOG_ECOM_0 -- Message 04 SBG_ECOM_LOG_MAG -- <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#SBG_ECOM_LOG_MAG>
/// Struct for the SBG_ECOM_LOG_MAG message
#[repr(C, packed)]
    #[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct SbgEcomLogMag {
    /// Time since sensor is powered up in µs
    pub time_stamp: u32,
    /// Magnetometer status bitmask, <https://developer.sbg-systems.com/sbgECom/5.3/binary_messages.html#MAG_STATUS>
    pub mag_status: u16,
    /// Magnetic field along the X axis in the body frame (Arbitrary Units)
    pub mag_x: f32,
    /// Magnetic field along the Y axis in the body frame (Arbitrary Units)
    pub mag_y: f32,
    /// Magnetic field along the Z axis in the body frame (Arbitrary Units)
    pub mag_z: f32,
    /// Acceleration along the X axis in the body frame (ms^-2) 
    pub accel_x: f32,
    /// Acceleration along the Y axis in the body frame (ms^-2) 
    pub accel_y: f32,
    /// Acceleration along the Z axis in the body frame (ms^-2) 
    pub accel_z: f32,
}