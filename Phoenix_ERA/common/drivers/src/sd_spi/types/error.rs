use embedded_sdmmc::{Error, SdCardError as _SdCardError};
pub type SdCardError = Error<_SdCardError>;
