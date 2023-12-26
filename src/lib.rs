#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![deny(rust_2018_idioms, unsafe_code)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![deny(clippy::unwrap_used)]

pub mod args;
pub mod config;
pub mod delimit;
pub mod log;
pub mod values;
