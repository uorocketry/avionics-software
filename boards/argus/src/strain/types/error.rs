use defmt::Format;
use derive_more::From;
use peripheral_services::serial::service::UsartError;

use crate::adc::service::AdcError;
use crate::sd::types::SdCardError;

#[derive(Debug, Format, From)]
pub enum StrainServiceError {
	AdcError(AdcError),
	UsartError(UsartError),
	SdCardError(SdCardError),
}
