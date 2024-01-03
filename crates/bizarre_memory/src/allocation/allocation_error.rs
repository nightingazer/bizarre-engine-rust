use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllocationError {
    #[error("Not enough memory to allocate {requested} bytes, only {available} bytes available")]
    OutOfMemory { requested: usize, available: usize },
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("Zero-sized allocation is requested from an allocator not capable of it")]
    ZeroSizedAllocation,
}
