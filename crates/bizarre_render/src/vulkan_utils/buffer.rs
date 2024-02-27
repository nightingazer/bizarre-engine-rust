use anyhow::Result;
use ash::vk;

use crate::global_context::VULKAN_GLOBAL_CONTEXT;

use super::vulkan_memory::find_memory_type_index;

pub fn create_buffer(
    size: usize,
    usage: vk::BufferUsageFlags,
    memory_flags: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let device = VULKAN_GLOBAL_CONTEXT.device();
    let memory_props = VULKAN_GLOBAL_CONTEXT.memory_properties();

    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .build();

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = find_memory_type_index(&memory_requirements, memory_flags).ok_or(
        anyhow::anyhow!("Failed to find suitable memory type for buffer allocation"),
    )?;

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .memory_type_index(memory_type_index)
        .allocation_size(memory_requirements.size);

    let buffer_memory = unsafe { device.allocate_memory(&allocate_info, None)? };

    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0)? };

    Ok((buffer, buffer_memory))
}
