#![no_std]
#![no_main]

//! This crate contains common code. Any code that is not platform specific should be put in
//! here.

pub mod drivers;
mod error;

use defmt_rtt as _;

pub use crate::error::error_manager::ErrorManager;
pub use crate::error::hydra_error::{ErrorContextTrait, HydraError, SpawnError}; // global logger
