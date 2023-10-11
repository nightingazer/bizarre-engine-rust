use thiserror::Error;

#[derive(Debug, Error)]
#[error("Suitability error: {0}.")]
pub struct SuitabilityError(pub &'static str);
