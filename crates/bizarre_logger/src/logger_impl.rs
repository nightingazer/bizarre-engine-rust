use std::{fs::OpenOptions, io::Write};

use anyhow::Result;

use crate::{
    escape_sequence, log_errors::LogError, log_level::LogLevel, TerminalEscapeSequence, RESET,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogTarget {
    Stdout,
    Stderr,
    File(&'static str),
}

#[derive(Debug)]
pub struct Logger {
    pub min_level: LogLevel,
    pub label: &'static str,
    pub targets: Vec<LogTarget>,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            min_level: LogLevel::Debug,
            label: "System",
            targets: vec![LogTarget::Stdout, LogTarget::Stderr],
        }
    }
}

impl Logger {
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
            LogTarget::File(path) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)
                    .map_err(|e| LogError::CouldNotOpenFile {
                        path: path.to_string(),
                        source: e.into(),
                    })?;

                let msg = format!("{} [{}]: {}\n", self.label, level, msg);
                Ok(file.write(msg.as_bytes()).map(|_| ())?)
            }
        }
    }
}
