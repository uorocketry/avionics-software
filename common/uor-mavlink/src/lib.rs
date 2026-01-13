#![cfg_attr(not(feature = "std"), no_std)]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "communications")]
pub mod uor_mavlink_communications;
#[cfg(feature = "traits")]
pub mod uor_mavlink_communications_traits;
#[cfg(feature = "dialect")]
pub mod uor_mavlink_dialect;
#[cfg(feature = "service")]
pub mod uor_mavlink_service;
