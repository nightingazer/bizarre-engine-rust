use anyhow::Result;

pub trait RawAllocator {
    fn alloc_raw(&mut self, size: usize, align: usize) -> Result<*mut u8>;
}

/// A contractual trait for allocators that won't move allocated objects in no
/// circumstances
pub trait StableAllocator {}

/// A trait for allocators that can allocate objects of any type.
/// This trait has a default implementation for all [RawAllocators](RawAllocator),
/// as it is possible to allocate objects of any type using raw allocation.
pub trait Allocator {
    fn alloc<T>(&mut self) -> Result<*mut T>;
}

impl<R: RawAllocator + Sized + 'static> Allocator for R {
    fn alloc<T>(&mut self) -> Result<*mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = self.alloc_raw(size, align)?.cast::<T>();
        unsafe { std::ptr::write(ptr, std::mem::zeroed()) };
        Ok(ptr)
    }
}

/// A trait for allocators that can allocate slices of any type.
///
pub trait SliceAllocator {
    fn alloc_slice<T>(&mut self, len: usize) -> Result<*mut [T]>;
}

impl<R: RawAllocator + Sized + 'static> SliceAllocator for R {
    fn alloc_slice<T>(&mut self, len: usize) -> Result<*mut [T]> {
        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();
        let ptr = self.alloc_raw(size, align)?.cast::<T>();
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
        Ok(slice)
    }
}

pub trait Constructor {
    fn construct<T: 'static>(&mut self, value: T) -> Result<*mut T>;
    fn construct_slice<T: 'static>(&mut self, values: &[T]) -> Result<*mut [T]>;
}

impl<R: RawAllocator + 'static> Constructor for R {
    fn construct<T>(&mut self, value: T) -> Result<*mut T> {
        let ptr = self.alloc::<T>()?;
        unsafe { std::ptr::write(ptr, value) }
        Ok(ptr)
    }

    fn construct_slice<T>(&mut self, values: &[T]) -> Result<*mut [T]> {
        let ptr = self.alloc_slice::<T>(values.len())?;
        unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), ptr.as_mut_ptr(), values.len()) };
        Ok(ptr)
    }
}

pub trait Deallocator {
    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn dealloc_raw(&mut self, ptr: *mut u8, size: usize, align: usize) -> Result<()>;

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn dealloc<T>(&mut self, ptr: *mut T) -> Result<()> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        unsafe { std::ptr::drop_in_place(ptr) };
        self.dealloc_raw(ptr.cast::<u8>(), size, align)
    }

    /// .
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    ///
    /// # Safety
    ///
    /// .
    unsafe fn dealloc_slice<T>(&mut self, ptr: *mut [T]) -> Result<()> {
        let size = std::mem::size_of::<T>() * ptr.len();
        let align = std::mem::align_of::<T>();
        unsafe { std::ptr::drop_in_place(ptr) };
        self.dealloc_raw(ptr.cast::<u8>(), size, align)
    }
}
