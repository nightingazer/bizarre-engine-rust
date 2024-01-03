use super::{
    allocation_error::AllocationError,
    allocator::{RawAllocator, StableAllocator},
    arena_chunk::ArenaChunk,
};

pub struct ArenaAllocator {
    chunks: Vec<ArenaChunk>,
    chunk_size: usize,
}

impl StableAllocator for ArenaAllocator {}

impl ArenaAllocator {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: vec![ArenaChunk::new(chunk_size)],
            chunk_size,
        }
    }

    pub fn reset(&mut self) {
        for chunk in self.chunks.iter_mut() {
            chunk.reset();
        }
    }
}

impl RawAllocator for ArenaAllocator {
    fn alloc_raw(&mut self, size: usize, align: usize) -> anyhow::Result<*mut u8> {
        if size > self.chunk_size {
            anyhow::bail!(AllocationError::OutOfMemory {
                requested: size,
                available: self.chunk_size
            })
        }
        for chunk in self.chunks.iter_mut() {
            match chunk.alloc_raw(size, align) {
                Ok(ptr) => return Ok(ptr),
                Err(e) => match e.downcast::<AllocationError>() {
                    Ok(AllocationError::OutOfMemory { .. }) => continue,
                    Ok(e) => anyhow::bail!(e),
                    Err(e) => anyhow::bail!(e),
                },
            }
        }
        self.chunks.push(ArenaChunk::new(self.chunk_size));
        self.chunks.last_mut().unwrap().alloc_raw(size, align)
    }
}
