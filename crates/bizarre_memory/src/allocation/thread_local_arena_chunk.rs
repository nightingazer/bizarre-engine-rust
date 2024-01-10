use crate::{AllocationError, RawAllocator, StableAllocator};

use super::arena_chunk::ArenaChunk;

/// A chunk of memory allocated through [ArenaAllocator](crate::ArenaAllocator) or [TypedArena](crate::TypedArena).
/// Although can be used by itself, as it implements [RawAllocator] trait, in
/// case when there is no need to dynamically allocate new chunks when there is
/// not enough memory inside an arena
pub struct ThreadLocalArenaChunk {
    start: *mut u8,
    end: *mut u8,
    ptr: *mut u8,
}

impl StableAllocator for ThreadLocalArenaChunk {}

impl ArenaChunk for ThreadLocalArenaChunk {
    fn new(size: usize) -> Self {
        let layout = std::alloc::Layout::from_size_align(size, 1).unwrap();
        let start = unsafe { std::alloc::alloc(layout) };
        let end = unsafe { start.add(size) };
        Self {
            start,
            end,
            ptr: end,
        }
    }

    fn reset(&mut self) {
        self.ptr = self.end;
    }

    fn free_arena(&mut self) {
        println!("Freeing arena chunk!");
        let layout =
            std::alloc::Layout::from_size_align(self.end as usize - self.start as usize, 1)
                .unwrap();
        unsafe { std::alloc::dealloc(self.start, layout) }
    }

    fn start(&self) -> *mut u8 {
        self.start
    }

    fn end(&self) -> *mut u8 {
        self.end
    }

    fn bump_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for ThreadLocalArenaChunk {
    fn drop(&mut self) {
        self.free_arena();
    }
}

impl RawAllocator for ThreadLocalArenaChunk {
    fn alloc_raw(&mut self, size: usize, align: usize) -> anyhow::Result<*mut u8> {
        debug_assert!(align > 0);
        debug_assert!(align.is_power_of_two());
        let ptr = self.ptr as usize;
        let new_ptr = ptr - size;
        let start = self.start as usize;
        if new_ptr < start {
            anyhow::bail!(AllocationError::OutOfMemory {
                requested: size,
                available: self.end as usize - self.start as usize
            });
        }

        let new_ptr = new_ptr & !(align - 1);
        if new_ptr < start {
            anyhow::bail!(AllocationError::OutOfMemory {
                requested: size,
                available: self.end as usize - self.start as usize
            });
        }

        self.ptr = new_ptr as *mut u8;
        Ok(self.ptr)
    }
}
