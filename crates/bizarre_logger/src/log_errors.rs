use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogError {
    #[error("The logger {0} is already initialized")]
    AlreadyInitialized(String),

    #[error("could not open the file '{path}': {source}")]
    CouldNotOpenFile { path: String, source: anyhow::Error },

    #[error("could not print to file '{path}': {source}")]
    CouldNotPrintToFile { path: String, source: anyhow::Error },
}
