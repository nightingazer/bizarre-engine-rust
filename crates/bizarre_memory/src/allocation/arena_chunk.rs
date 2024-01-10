use crate::allocation::allocation_error::AllocationError;

use super::allocator::{RawAllocator, StableAllocator};

pub trait ArenaChunk: RawAllocator + StableAllocator {
    fn new(size: usize) -> Self;
    fn reset(&mut self);
    fn free_arena(&mut self);
    fn start(&self) -> *mut u8;
    fn end(&self) -> *mut u8;
    fn bump_ptr(&self) -> *mut u8;
}
