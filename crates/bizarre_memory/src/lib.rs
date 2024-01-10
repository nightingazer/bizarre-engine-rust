#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]

pub mod allocation;
pub use allocation::{
    allocation_error::AllocationError, allocator::*, arena::ArenaAllocator,
    deallocation_error::DeallocationError, sync_arena_chunk::SyncArenaChunk,
    thread_local_arena_chunk::ThreadLocalArenaChunk, typed_arena::TypedArena,
};
