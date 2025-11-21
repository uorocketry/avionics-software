use defmt::Format;
use derive_more::From;
use serial::service::UsartError;

use crate::adc::service::AdcError;
use crate::sd::types::SdCardError;

#[derive(Debug, Format, From)]
pub enum PressureServiceError {
	AdcError(AdcError),
	UsartError(UsartError),
	SdCardError(SdCardError),
	FormatError,
}
