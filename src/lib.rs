#![forbid(unsafe_code)]

#![deny(warnings)]
#![deny(clippy::pedantic)]
#![allow(clippy::explicit_deref_methods)]
#![allow(clippy::module_name_repetitions)]

#![doc = include_str!("../README.md")]



mod constants;
mod error;
mod generator;

pub use constants::DEFAULT_EPOCH;
pub use error::*;
pub use generator::Generator;

