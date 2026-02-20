#![no_std]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "music")]
pub mod music;
pub mod sound;
pub mod utils;
#[cfg(feature = "led_indicator")]
pub mod led_indicator;