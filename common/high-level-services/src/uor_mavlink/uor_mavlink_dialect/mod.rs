// #![cfg_attr(not(feature = "std"), no_std)]
// // include generate definitions
include!("./output/mod.rs");

// TODO: The mavlink-bindgen crate discusses a method of using it where you don't need to keep doing mavlink-bindgen, change method to that:
// https://github.com/mavlink/rust-mavlink/tree/master/mavlink-bindgen
pub use mavlink_core::*;
