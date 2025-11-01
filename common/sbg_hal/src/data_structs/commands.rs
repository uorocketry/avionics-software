use bytemuck::{NoUninit}; 



pub trait SbgCommand{
    fn msg_number(self) -> u8;
}

// SBG_ECOM_CLASS_CMD_0 -- Command 05 SBG_ECOM_CMD_INIT_PARAMETERS -- FOUND IN <https://support.sbg-systems.com/sc/dev/latest/firmware-documentation>, VERSION 2.3, PG#: 31 
/// Struct for the SbgEcomCmdInitParameters command
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, NoUninit)]
pub struct SbgEcomCmdInitParameters {
    /// Initial latitude in degrees
    pub init_lad: f64,
    /// Initial longitude in degrees
    pub init_long: f64,
    /// Initial altitude in meters (above WGS84 ellipsoid)
    pub init_alt: f64,
    /// Year at startup
    pub year: u16,
    /// Month at startup
    pub month: u8,
    /// Day at startup
    pub day: u8
}
impl SbgCommand for SbgEcomCmdInitParameters{
    fn msg_number(self) -> u8 {
        5
    }
}