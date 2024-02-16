use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Mutex, MutexGuard,
};

use crate::{AllocationError, RawAllocator, StableAllocator};

use super::arena_chunk::ArenaChunk;

pub struct SyncArenaChunk {
    start: AtomicPtr<u8>,
    end: AtomicPtr<u8>,
    ptr: Mutex<AtomicPtr<u8>>,
}

unsafe impl Sync for SyncArenaChunk {}
unsafe impl Send for SyncArenaChunk {}

impl StableAllocator for SyncArenaChunk {}

impl SyncArenaChunk {
    pub fn lock_bump_ptr(&self) -> anyhow::Result<MutexGuard<'_, AtomicPtr<u8>>> {
        let ptr_lock = self.ptr.lock().unwrap();
        Ok(ptr_lock)
    }
}

impl RawAllocator for SyncArenaChunk {
    fn alloc_raw(&mut self, size: usize, align: usize) -> anyhow::Result<*mut u8> {
        let ptr_lock = self.lock_bump_ptr()?;

        debug_assert!(align > 0);
        debug_assert!(align.is_power_of_two());
        let ptr = ptr_lock.load(Ordering::SeqCst) as usize;
        let new_ptr = ptr - size;
        let start = self.start.load(Ordering::SeqCst);
        let end = self.end.load(Ordering::SeqCst);
        if new_ptr < start as usize {
            anyhow::bail!(AllocationError::OutOfMemory {
                requested: size,
                available: end as usize - start as usize
            });
        }

        let new_ptr = new_ptr & !(align - 1);
        if new_ptr < start as usize {
            anyhow::bail!(AllocationError::OutOfMemory {
                requested: size,
                available: end as usize - start as usize
            });
        }

        ptr_lock.store(new_ptr as *mut u8, Ordering::SeqCst);
        Ok(new_ptr as *mut u8)
    }
}

impl ArenaChunk for SyncArenaChunk {
    fn new(size: usize) -> Self {
        let memory =
            unsafe { std::alloc::alloc(std::alloc::Layout::from_size_align(size, 1).unwrap()) };
        let start = memory;
        let end = unsafe { start.add(size) };

        Self {
            start: AtomicPtr::new(start),
            end: AtomicPtr::new(end),
            ptr: Mutex::new(AtomicPtr::new(end)),
        }
    }

    fn reset(&mut self) {
        let ptr_lock = self.ptr.lock().unwrap();
        let bump = self.end.load(std::sync::atomic::Ordering::SeqCst);
        ptr_lock.store(bump, std::sync::atomic::Ordering::SeqCst);
    }

    fn free_arena(&mut self) {
        let start = self.start.load(std::sync::atomic::Ordering::SeqCst);
        let end = self.end.load(std::sync::atomic::Ordering::SeqCst);
        unsafe {
            std::alloc::dealloc(
                start,
                std::alloc::Layout::from_size_align(end as usize - start as usize, 1).unwrap(),
            )
        };
    }

    fn start(&self) -> *mut u8 {
        self.start.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn end(&self) -> *mut u8 {
        self.end.load(std::sync::atomic::Ordering::SeqCst)
    }

    fn bump_ptr(&self) -> *mut u8 {
        self.ptr.lock().unwrap().load(Ordering::SeqCst)
    }
}
