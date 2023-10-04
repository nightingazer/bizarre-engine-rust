use std::fmt::Display;

#[derive(Debug)]
pub enum LogError {
    AlreadyInitialized,
    CouldNotOpenFile { path: String, reason: String },
    CouldNotPrintToFile { path: String, reason: String },
}

impl Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialized => write!(f, "already initialized"),
            Self::CouldNotOpenFile { path, reason } => {
                write!(f, "could not open file '{path}': {reason}")
            }
            Self::CouldNotPrintToFile { path, reason } => {
                write!(f, "could not write to file '{path}': {reason}")
            }
        }
    }
}
