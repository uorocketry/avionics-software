use defmt::Format;

use crate::adc::service::AdcError;
use crate::sd::types::SdCardError;
use crate::serial::service::UsartError;

#[derive(Debug, Format)]
pub enum TemperatureServiceError {
	AdcError(AdcError),
	UsartError(UsartError),
	SdCardError(SdCardError),
}
impl From<AdcError> for TemperatureServiceError {
	fn from(err: AdcError) -> Self {
		TemperatureServiceError::AdcError(err)
	}
}
impl From<UsartError> for TemperatureServiceError {
	fn from(err: UsartError) -> Self {
		TemperatureServiceError::UsartError(err)
	}
}
impl From<SdCardError> for TemperatureServiceError {
	fn from(err: SdCardError) -> Self {
		TemperatureServiceError::SdCardError(err)
	}
}
