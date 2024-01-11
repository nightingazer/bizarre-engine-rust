use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::Write,
};

use anyhow::Result;

use crate::{
    escape_sequence,
    log_errors::LogError,
    log_level::LogLevel,
    log_target::{file_target, LogTarget},
    TerminalEscapeSequence, RESET,
};

pub struct LogMessage {
    pub level: LogLevel,
    pub msg: String,
    pub logger_name: &'static str,
    pub shutdown: bool,
}

#[derive(Debug)]
pub struct Logger {
    min_level: LogLevel,
    label: &'static str,
    name: &'static str,
    targets: Vec<LogTarget>,
}

pub const CORE_LOGGER_NAME: &str = "core";
pub const APP_LOGGER_NAME: &str = "app";

impl Default for Logger {
    fn default() -> Self {
        Self::new(
            LogLevel::Debug,
            "Logger",
            "logger",
            vec![LogTarget::Stdout, LogTarget::Stderr],
        )
    }
}

impl Logger {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn default_core() -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S");

        Self::new(
            LogLevel::Debug,
            "Engine",
            CORE_LOGGER_NAME,
            vec![
                LogTarget::Stdout,
                LogTarget::Stderr,
                file_target(
                    format!("log/{}_{}.log", CORE_LOGGER_NAME, timestamp).as_str(),
                    None,
                ),
            ],
        )
    }
    pub fn default_app() -> Self {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H:%M:%S");

        Self::new(
            LogLevel::Debug,
            "App",
            APP_LOGGER_NAME,
            vec![
                LogTarget::Stdout,
                LogTarget::Stderr,
                file_target(
                    format!("log/{}_{}.log", APP_LOGGER_NAME, timestamp).as_str(),
                    None,
                ),
            ],
        )
    }

    pub fn new(
        min_level: LogLevel,
        label: &'static str,
        name: &'static str,
        targets: Vec<LogTarget>,
    ) -> Self {
        let targets = targets.into_iter().map(|mut t| {
            if let LogTarget::File(path, file) = &mut t {
                if file.is_none() {
                    let dir = std::path::Path::new(path.as_ref()).parent().unwrap();
                    if dir.is_absolute() {
                        panic!("The path to the log file must be relative");
                    }
                    if !dir.exists() {
                        std::fs::create_dir_all(dir).unwrap();
                    }
                    let opened_file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(path.as_ref())
                        .expect(format!("Failed to open the log file: {}", path).as_str());
                    *file = Some(RefCell::new(opened_file));
                }
            }
            t
        });

        Self {
            min_level,
            label,
            name,
            targets: targets.collect(),
        }
    }

    pub fn log(&self, level: LogLevel, msg: String) {
        if level < self.min_level {
            return;
        }

        for target in self.targets.iter() {
            if let Err(e) = self.log_to_target(target, &level, &msg) {
                eprintln!("{e}");
            }
        }
    }

    fn log_to_target(
        &self,
        target: &LogTarget,
        level: &LogLevel,
        msg: &String,
    ) -> Result<(), anyhow::Error> {
        if (target != &LogTarget::Stderr && level >= &LogLevel::Error)
            || (target == &LogTarget::Stderr && level < &LogLevel::Error)
        {
            return Ok(());
        }
        match target {
            LogTarget::Stderr | LogTarget::Stdout => {
                let msg = format!(
                    "{}{} [{}]: {}{}",
                    TerminalEscapeSequence::from(level),
                    self.label,
                    level,
                    msg,
                    escape_sequence!(RESET)
                );
                if target == &LogTarget::Stdout {
                    println!("{msg}");
                    Ok(())
                } else {
                    eprintln!("{msg}");
                    Ok(())
                }
            }
            LogTarget::File(_, Some(file)) => {
                let msg = format!("{} [{}]: {}\n", self.label, level, msg);
                Ok(file.try_borrow_mut()?.write(msg.as_bytes()).map(|_| ())?)
            }
            _ => panic!("LogTarget::File(_, None) is not allowed after the logger initialization"),
        }
    }
}
