#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

pub mod adc;
pub mod led_indicator;
pub mod linear_transformation;
pub mod node;
pub mod sd;
pub mod serial;
pub mod session;
pub mod state_machine;
pub mod strain;
pub mod utils;

#[cfg(feature = "temperature")]
pub mod temperature;

#[cfg(feature = "pressure")]
pub mod pressure;
