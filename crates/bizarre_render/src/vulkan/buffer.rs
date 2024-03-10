use std::{
    marker::PhantomData,
    mem::{align_of, size_of},
    ops::{
        Bound::{Excluded, Included, Unbounded},
        RangeBounds,
    },
};

use anyhow::Result;
use ash::{util::Align, vk};
use bizarre_logger::core_warn;
use thiserror::Error;

use crate::vulkan_utils::buffer::create_buffer;

use super::device::VulkanDevice;

pub struct VulkanBuffer<T> {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    _phantom: PhantomData<T>,
}

impl<T> VulkanBuffer<T> {
    const SIZE: usize = size_of::<T>();
}

impl<T> VulkanBuffer<T> {
    pub fn new(
        usage: vk::BufferUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
        device: &VulkanDevice,
    ) -> Result<Self> {
        let (buffer, memory) = create_buffer(Self::SIZE, usage, memory_flags, device)?;
        Ok(Self {
            buffer,
            memory,
            _phantom: Default::default(),
        })
    }

    pub fn map_memory(&self, device: &VulkanDevice) -> Result<Box<T>> {
        let ptr = unsafe {
            device
                .map_memory(
                    self.memory,
                    0,
                    Self::SIZE as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast()
        };

        let boxed = unsafe { Box::from_raw(ptr) };
        Ok(boxed)
    }

    pub fn unmap_memory(&self, ptr: Box<T>, device: &VulkanDevice) {
        unsafe { device.unmap_memory(self.memory) }
        drop(ptr);
    }
}

pub struct VulkanSliceBuffer<T> {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> Default for VulkanSliceBuffer<T> {
    fn default() -> Self {
        Self {
            buffer: vk::Buffer::null(),
            memory: vk::DeviceMemory::null(),
            capacity: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<T> VulkanSliceBuffer<T> {
    const ELEMENT_SIZE: usize = size_of::<T>();
}

impl<T> VulkanSliceBuffer<T> {
    pub fn new(
        capacity: usize,
        usage: vk::BufferUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
        device: &VulkanDevice,
    ) -> Result<Self> {
        let (buffer, memory) =
            create_buffer(Self::ELEMENT_SIZE * capacity, usage, memory_flags, device)?;

        Ok(Self {
            buffer,
            memory,
            capacity,
            _phantom: Default::default(),
        })
    }

    pub fn map_offset_count(
        &self,
        offset: usize,
        count: usize,
        device: &VulkanDevice,
    ) -> Result<Align<T>> {
        let offset = offset * Self::ELEMENT_SIZE;
        let size = count * Self::ELEMENT_SIZE;

        let ptr = unsafe {
            device.map_memory(
                self.memory,
                offset as u64,
                size as u64,
                vk::MemoryMapFlags::empty(),
            )?
        };

        let align = unsafe { Align::<T>::new(ptr, align_of::<T>() as u64, size as u64) };

        Ok(align)
    }

    pub fn map_range<R: RangeBounds<usize>>(
        &self,
        range: R,
        device: &VulkanDevice,
    ) -> Result<Align<T>> {
        let first_index = match range.start_bound() {
            Included(index) => *index,
            Excluded(index) => index + 1,
            Unbounded => 0,
        };
        let last_index = match range.end_bound() {
            Included(index) => *index + 1,
            Excluded(index) => *index,
            Unbounded => self.capacity,
        };

        let offset = (first_index * Self::ELEMENT_SIZE) as vk::DeviceSize;
        let size = (last_index - first_index * Self::ELEMENT_SIZE) as vk::DeviceSize;

        let ptr =
            unsafe { device.map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())? };

        let align = unsafe { Align::<T>::new(ptr, align_of::<T>() as u64, size) };

        Ok(align)
    }

    pub fn unmap_memory(&self, align: Align<T>, device: &VulkanDevice) {
        unsafe { device.unmap_memory(self.memory) };
        drop(align)
    }
}
