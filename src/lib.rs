#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
// I believe that return statements easier to follow.
#![allow(clippy::needless_return)]
#![allow(clippy::module_name_repetitions)]

//! # HexaFreeze
#![doc = include_str!("../README.md")]

mod util;
mod generator;

pub use generator::Generator;
pub use generator::GeneratorError;

