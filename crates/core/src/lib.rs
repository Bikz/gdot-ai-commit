#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::struct_excessive_bools
)]

pub mod config;
pub mod diff;
pub mod error;
pub mod git;
pub mod ignore;
pub mod pipeline;
pub mod prompt;
pub mod providers;
pub mod retry;
