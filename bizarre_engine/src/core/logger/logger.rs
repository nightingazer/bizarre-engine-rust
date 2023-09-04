use std::{collections::HashMap, sync::Mutex};

use super::log_level::LogLevel;

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
    target: LogTarget,
}

#[derive(Debug)]
pub struct LoggerBuilder {
    min_level: LogLevel,
    tag: String,
    format: String,
    target: LogTarget,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            min_level: LogLevel::Debug,
            tag: String::new(),
            format: String::new(),
            target: LogTarget::Stdout,
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

    pub fn target(mut self, target: LogTarget) -> Self {
        self.target = target;
        self
    }

    pub fn build() -> Logger {
        Logger {
            min_level: LogLevel::Debug,
            tag: String::new(),
            format: String::new(),
            target: LogTarget::Stdout,
        }
    }
}

static mut logger_map: Option<Mutex<HashMap<String, Logger>>> = None;

pub fn init_logging() {
    unsafe {
        logger_map = Some(Mutex::new(HashMap::new()));
    }
}

pub fn register_logger(logger: Logger, label: &str) {
    unsafe {
        let mut loggers = logger_map.as_ref().unwrap().lock().unwrap();
    }
}
