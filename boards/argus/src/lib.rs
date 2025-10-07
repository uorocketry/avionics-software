#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

pub mod adc;
pub mod linear_transformation;
pub mod sd;
pub mod serial;
pub mod state_machine;
pub mod utils;

#[cfg(feature = "temperature")]
pub mod temperature;

#[cfg(feature = "pressure")]
pub mod pressure;
