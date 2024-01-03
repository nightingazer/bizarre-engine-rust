use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeallocationError {
    /// Can be thrown by an allocator if the pointer was not allocated by it
    #[error("Trying to deallocate a pointer that was not allocated by this allocator")]
    NotFromAllocator,
}
