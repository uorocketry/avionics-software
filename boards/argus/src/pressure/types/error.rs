use defmt::Format;
use derive_more::From;

use crate::adc::service::AdcError;
use crate::sd::types::SdCardError;
use crate::serial::service::UsartError;

#[derive(Debug, Format, From)]
pub enum PressureServiceError {
	AdcError(AdcError),
	UsartError(UsartError),
	SdCardError(SdCardError),
}
