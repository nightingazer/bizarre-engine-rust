#![feature(macro_metavar_expr)]

pub mod global_loggers;
pub mod log_errors;
pub mod log_level;
pub mod logger_impl;
pub mod terminal_escape_code;
pub mod terminal_macros;

pub use global_loggers::{app_logger_init, core_logger_init};
pub use log_level::*;
pub use terminal_escape_code::*;
