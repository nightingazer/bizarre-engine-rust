use std::{collections::HashMap, sync::Mutex};

use super::{
    log_level::LogLevel,
    terminal_escape_code::{TerminalEscapeCode, TerminalEscapeSequence, RESET},
};

#[derive(Debug, Clone)]
pub enum LogTarget {
    Stdout,
    Stderr,
    File(String),
}

#[derive(Debug)]
pub struct Logger {
    min_level: LogLevel,
    tag: String,
    format: String,
    target: Vec<LogTarget>,
}

#[derive(Debug)]
pub struct LoggerBuilder {
    min_level: LogLevel,
    tag: String,
    format: String,
    target: Vec<LogTarget>,
}

const DEFAULT_FORMAT: &str = "{c_start}{time} - {tag} [{level}]: {message}{c_stop}";

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            min_level: LogLevel::Debug,
            tag: String::new(),
            format: String::from(DEFAULT_FORMAT),
            target: vec![LogTarget::Stdout],
        }
    }

    pub fn min_level(mut self, min_level: LogLevel) -> Self {
        self.min_level = min_level;
        self
    }

    pub fn tag(mut self, tag: &str) -> Self {
        self.tag = tag.to_string();
        self
    }

    pub fn format(mut self, format: &str) -> Self {
        self.format = format.to_string();
        self
    }

    pub fn target(mut self, target: Vec<LogTarget>) -> Self {
        self.target = target;
        self
    }

    pub fn build(&self) -> Logger {
        Logger {
            min_level: self.min_level.clone(),
            tag: self.tag.clone(),
            format: self.format.clone(),
            target: self.target.clone(),
        }
    }
}

impl Logger {
    pub fn log(&self, level: LogLevel, message: String) {
        if level < self.min_level {
            return;
        }

        let log = self
            .format
            .clone()
            .replace("{level}", &level.to_string())
            .replace("{message}", &message)
            .replace("{tag}", &self.tag)
            .replace(
                "{c_start}",
                TerminalEscapeSequence::from(level).to_string().as_str(),
            )
            .replace(
                "{c_stop}",
                TerminalEscapeSequence::from(RESET).to_string().as_str(),
            );
        for target in &self.target {
            match target {
                LogTarget::Stdout => println!("{}", log),
                LogTarget::Stderr => eprintln!("{}", log),
                LogTarget::File(path) => todo!(),
            }
        }
    }
}

pub static mut CORE_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub fn init_core_logger(logger: Option<Logger>) {
    let logger = match logger {
        Some(logger) => logger,
        None => Logger {
            min_level: LogLevel::Debug,
            tag: String::from("Engine"),
            format: String::from(DEFAULT_FORMAT),
            target: vec![LogTarget::Stdout],
        },
    };

    unsafe {
        *CORE_LOGGER.lock().unwrap() = Some(logger);
    }
}

pub static mut APP_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub fn init_app_logger(logger: Option<Logger>) {
    let logger = match logger {
        Some(logger) => logger,
        None => Logger {
            min_level: LogLevel::Debug,
            tag: String::from("App"),
            format: String::from(DEFAULT_FORMAT),
            target: vec![LogTarget::Stdout],
        },
    };

    unsafe {
        *APP_LOGGER.lock().unwrap() = Some(logger);
    }
}

#[macro_export]
macro_rules! log_to_global {
    ($logger_mut:expr, $level:expr, $message:expr) => {
        let logger = unsafe { $logger_mut.lock().unwrap() };

        match & *logger {
            Some(logger) => logger.log($level, $message.to_string()),
            None => {
                panic!("Tried to log to a logger that was not initialized!")
            }
        }
    };

    ($logger_mut:expr, $level:expr, $fmt:expr, $($arg:tt),+) => {
        let logger = unsafe { $logger_mut.lock().unwrap() };

        match logger {
            Some(logger) => logger.log($level, format!($fmt, $($arg)*)),
            None => {
                panic!("Tried to log to a logger that was not initialized!")
            }
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt),+) => {
        log_to_global!(&APP_LOGGER, LogLevel::Debug, $($arg),+)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt),+) => {
        log_to_global!(&APP_LOGGER, LogLeel::I)
    };
}
