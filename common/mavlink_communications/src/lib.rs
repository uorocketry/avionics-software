#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod macros;
pub mod publishers;
pub mod subscribers;
// Importing derive macros
use mavlink_communications_macros::{Publisher, Subscriber};
