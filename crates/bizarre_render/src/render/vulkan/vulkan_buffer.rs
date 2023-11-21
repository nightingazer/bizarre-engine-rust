use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateFlags, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocatePreference, MemoryAllocator, MemoryTypeFilter,
    },
};

pub fn create_ubo<T: BufferContents>(
    allocator: Arc<dyn MemoryAllocator>,
    data: T,
) -> Result<Subbuffer<T>> {
    let buffer = Buffer::from_data(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            // allocate_preference: MemoryAllocatePreference::AlwaysAllocate,
            ..Default::default()
        },
        data,
    )?;

    Ok(buffer)
}

pub fn create_vbo<T, I>(allocator: Arc<dyn MemoryAllocator>, iter: I) -> Result<Subbuffer<[T]>>
where
    T: BufferContents,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let buffer = Buffer::from_iter(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        iter,
    )?;

    Ok(buffer)
}

pub fn create_ibo<I>(allocator: Arc<dyn MemoryAllocator>, data: I) -> Result<Subbuffer<[u32]>>
where
    I: IntoIterator<Item = u32>,
    I::IntoIter: ExactSizeIterator,
{
    let buffer = Buffer::from_iter(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        data,
    )?;

    Ok(buffer)
}
