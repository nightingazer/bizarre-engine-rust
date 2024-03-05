use anyhow::Result;
use ash::vk;

use crate::vulkan::device::VulkanDevice;

pub fn create_buffer(
    size: usize,
    usage: vk::BufferUsageFlags,
    memory_flags: vk::MemoryPropertyFlags,
    device: &VulkanDevice,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size as u64)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .build();

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_type_index = device
        .find_memory_type_index(&memory_requirements, memory_flags)
        .ok_or(anyhow::anyhow!(
            "Failed to find suitable memory type for buffer allocation"
        ))?;

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .memory_type_index(memory_type_index)
        .allocation_size(memory_requirements.size);

    let buffer_memory = unsafe { device.allocate_memory(&allocate_info, None)? };

    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0)? };

    Ok((buffer, buffer_memory))
}
