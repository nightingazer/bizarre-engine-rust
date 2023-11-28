use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    image::view::ImageView,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    DeviceSize,
};

use crate::text::ScreenText;

use super::{vertex::VulkanVertex2D, vulkan_image::create_texture};

pub struct VulkanScreenTextObject {
    pub vertex_buffer: Subbuffer<[VulkanVertex2D]>,
    pub index_buffer: Subbuffer<[u32]>,
    pub index_count: usize,
    pub font_texture: Arc<ImageView>,
    pub len: usize,
    pub capacity: usize,
}

impl VulkanScreenTextObject {
    /// Creates an uninitialized screen text object with a capacity of given character count.
    pub fn with_capacity(
        capacity: usize,
        font_texture: Arc<ImageView>,
        mem_allocator: Arc<dyn MemoryAllocator>,
    ) -> Result<Self> {
        let vertex_buffer = Buffer::new_slice(
            mem_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                    | MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            capacity as u64 * 4,
        )?;

        let index_buffer = Buffer::new_slice(
            mem_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE
                    | MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            capacity as u64 * 6,
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
            index_count: 0,
            font_texture,
            capacity,
            len: 0,
        })
    }

    pub fn update_buffers(&self, vbo: &[VulkanVertex2D], ibo: &[u32]) -> Result<()> {
        unsafe {
            let vbo_ptr = self
                .vertex_buffer
                .clone()
                .slice(0..vbo.len() as DeviceSize)
                .mapped_slice()?
                .as_ptr();
            let ibo_ptr = self
                .index_buffer
                .clone()
                .slice(0..ibo.len() as DeviceSize)
                .mapped_slice()?
                .as_ptr();

            std::ptr::copy_nonoverlapping(vbo.as_ptr(), vbo_ptr as *mut _, vbo.len());
            std::ptr::copy_nonoverlapping(ibo.as_ptr(), ibo_ptr as *mut _, ibo.len());
        }

        Ok(())
    }
}
