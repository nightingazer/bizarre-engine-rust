#![feature(slice_ptr_len)]
#![feature(slice_ptr_get)]

pub mod allocation;
pub use allocation::{
    allocation_error::AllocationError, allocator::*, arena::ArenaAllocator,
    arena_chunk::ArenaChunk, deallocation_error::DeallocationError, typed_arena::TypedArena,
};
