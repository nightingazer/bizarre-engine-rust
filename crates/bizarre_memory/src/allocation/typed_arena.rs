use std::any::TypeId;

use anyhow::Result;

use crate::allocation::allocation_error::AllocationError;

use super::{
    allocator::{Constructor, RawAllocator, StableAllocator},
    arena_chunk::ArenaChunk,
};

/// An arena allocator that allocates objects of a single type.
/// All allocated objects will be dropped when the arena is dropped or reset.
/// Is not thread safe.
/// Can construct objects but cannot give a raw chunk of memory.
/// Works slower than the raw [`ArenaAllocator`](crate::ArenaAllocator),
/// so if there is no need to drop the allocated objects,
/// it is better to use the raw arena allocator.
pub struct TypedArena<T: 'static> {
    chunks: Vec<ArenaChunk>,
    chunk_capacity: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> StableAllocator for TypedArena<T> {}

impl<T: 'static> TypedArena<T> {
    pub fn new(chunk_capacity: usize) -> Self {
        Self {
            chunk_capacity,
            chunks: vec![ArenaChunk::new(chunk_capacity * std::mem::size_of::<T>())],
            _phantom: std::marker::PhantomData,
        }
    }

    /// Resets the arena, dropping all allocated objects.
    pub fn reset(&mut self) {
        for chunk in self.chunks.iter_mut() {
            let bump = chunk.bump_ptr() as *mut T;
            let end = chunk.end() as *mut T;
            if bump == end {
                continue;
            }
            let slice = unsafe {
                std::slice::from_raw_parts_mut(
                    bump,
                    (end as usize - bump as usize) / std::mem::size_of::<T>(),
                )
            };
            for el in slice.iter_mut() {
                unsafe { std::ptr::drop_in_place(el) }
            }
            chunk.reset();
        }
    }

    fn alloc_ptr(&mut self, size: usize, align: usize) -> Result<*mut u8> {
        if size > self.chunk_capacity * std::mem::size_of::<T>() {
            anyhow::bail!(super::allocation_error::AllocationError::OutOfMemory {
                requested: size,
                available: self.chunk_capacity * std::mem::size_of::<T>()
            })
        }
        for chunk in self.chunks.iter_mut() {
            match chunk.alloc_raw(size, align) {
                Ok(ptr) => return Ok(ptr),
                Err(e) => match e.downcast::<super::allocation_error::AllocationError>() {
                    Ok(super::allocation_error::AllocationError::OutOfMemory { .. }) => continue,
                    Ok(e) => anyhow::bail!(e),
                    Err(e) => anyhow::bail!(e),
                },
            }
        }
        self.chunks.push(ArenaChunk::new(
            self.chunk_capacity * std::mem::size_of::<T>(),
        ));
        self.chunks.last_mut().unwrap().alloc_raw(size, align)
    }

    #[cfg(debug_assertions)]
    fn assert_type<V: 'static>() -> Result<()> {
        if TypeId::of::<T>() != TypeId::of::<V>() {
            anyhow::bail!(super::allocation_error::AllocationError::TypeMismatch {
                expected: std::any::type_name::<T>(),
                actual: std::any::type_name::<V>(),
            })
        } else {
            Ok(())
        }
    }
}

impl<T: 'static> Constructor for TypedArena<T> {
    fn construct<V: 'static>(&mut self, value: V) -> Result<*mut V> {
        #[cfg(debug_assertions)]
        Self::assert_type::<V>()?;

        let ptr: *mut V = self
            .alloc_ptr(std::mem::size_of::<V>(), std::mem::align_of::<V>())?
            .cast::<V>();

        unsafe { ptr.write(value) };
        Ok(ptr)
    }

    fn construct_slice<V: 'static>(&mut self, values: &[V]) -> Result<*mut [V]> {
        #[cfg(debug_assertions)]
        {
            Self::assert_type::<T>()?;
            if values.is_empty() {
                anyhow::bail!(AllocationError::ZeroSizedAllocation)
            }
        }

        let ptr: *mut V = self
            .alloc_ptr(std::mem::size_of_val(values), std::mem::align_of::<V>())?
            .cast::<V>();

        unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len()) };
        Ok(unsafe { std::slice::from_raw_parts_mut(ptr, values.len()) })
    }
}

impl<T> Drop for TypedArena<T> {
    fn drop(&mut self) {
        self.reset();
    }
}

impl<T> Default for TypedArena<T> {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_typed_arena() -> Result<()> {
        let mut arena = TypedArena::<u32>::new(100);

        let u32_1 = arena.construct(1u32)?;
        let u32_2 = arena.construct(2u32)?;
        let u32_3 = arena.construct(3u32)?;

        assert!(arena.chunks.len() == 1, "Arena should have only one chunk");
        {
            let chunk = &arena.chunks[0];
            let bump_ptr = chunk.bump_ptr();
            let end = chunk.end();
            let filled = end as usize - bump_ptr as usize;
            assert!(
                filled >= 3 * std::mem::size_of::<u32>(),
                "Chunk should be filled with 3 u32s"
            );
        }

        assert_ne!(u32_1, std::ptr::null_mut());
        assert_ne!(u32_2, std::ptr::null_mut());
        assert_ne!(u32_3, std::ptr::null_mut());
        assert_eq!(unsafe { *u32_1 }, 1);
        assert_eq!(unsafe { *u32_2 }, 2);
        assert_eq!(unsafe { *u32_3 }, 3);

        arena.reset();

        let u32_4 = arena.construct(4u32)?;
        let u32_5 = arena.construct(5u32)?;
        let u32_6 = arena.construct(6u32)?;

        assert!(
            arena.chunks.len() == 1,
            "Arena should have only one chunk after reset"
        );
        {
            let chunk = &arena.chunks[0];
            let bump_ptr = chunk.bump_ptr();
            let end = chunk.end();
            let filled = end as usize - bump_ptr as usize;
            assert!(
                filled >= 3 * std::mem::size_of::<u32>(),
                "Chunk should be filled with 3 u32s after reset"
            );
        }

        assert_ne!(u32_4, std::ptr::null_mut());
        assert_ne!(u32_5, std::ptr::null_mut());
        assert_ne!(u32_6, std::ptr::null_mut());
        assert_eq!(unsafe { *u32_4 }, 4);
        assert_eq!(unsafe { *u32_5 }, 5);
        assert_eq!(unsafe { *u32_6 }, 6);

        Ok(())
    }

    #[test]
    fn test_typed_arena_slice() {
        let mut typed_arena = TypedArena::<u32>::new(100);

        let u32_slice = typed_arena.construct_slice(&[1, 2, 3]).unwrap();

        assert_eq!(unsafe { u32_slice.as_ref().unwrap() }, [1, 2, 3]);

        typed_arena.reset();

        let u32_slice = typed_arena.construct_slice(&[4, 5, 6]).unwrap();

        assert_eq!(unsafe { u32_slice.as_ref().unwrap() }, [4, 5, 6]);
    }

    #[test]
    #[should_panic(expected = "Type mismatch")]
    fn test_type_mismatch() {
        struct Foo;

        let mut typed_arena = TypedArena::<u32>::new(100);
        let _foo = typed_arena.construct(Foo).unwrap();
    }

    #[test]
    fn test_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Foo<'a>(&'a AtomicUsize);

        impl<'a> Drop for Foo<'a> {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);

        {
            let mut typed_arena = TypedArena::<Foo>::new(100);
            let _foo = typed_arena.construct(Foo(&DROP_COUNTER)).unwrap();
            assert_eq!(
                DROP_COUNTER.load(Ordering::SeqCst),
                0,
                "Foo should not be dropped yet"
            );
        }

        assert_eq!(
            DROP_COUNTER.load(Ordering::SeqCst),
            1,
            "Foo should be dropped"
        );
    }
}
