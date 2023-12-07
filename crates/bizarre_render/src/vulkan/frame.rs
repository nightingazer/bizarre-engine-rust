use anyhow::Result;
use ash::vk;

use crate::{
    vertex::ColorNormalVertex, vulkan_shaders::deferred, vulkan_utils::buffer::create_buffer,
};

pub struct VulkanFrame {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub deferred_set: vk::DescriptorSet,

    pub deferred_vbo: vk::Buffer,
    pub deferred_ibo: vk::Buffer,
    pub deferred_ubo: vk::Buffer,

    pub deferred_vbo_mem: vk::DeviceMemory,
    pub deferred_ibo_mem: vk::DeviceMemory,
    pub deferred_ubo_mem: vk::DeviceMemory,

    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
}

pub struct VulkanFrameInfo<'a> {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub memory_props: &'a vk::PhysicalDeviceMemoryProperties,
    pub uniform_descriptor_pool: vk::DescriptorPool,
}

impl VulkanFrame {
    pub fn new(info: &VulkanFrameInfo, device: &ash::Device) -> Result<Self> {
        let (image_available_semaphore, render_finished_semaphore) = unsafe {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let ia_semaphore = device.create_semaphore(&semaphore_create_info, None)?;
            let rf_semaphore = device.create_semaphore(&semaphore_create_info, None)?;

            (ia_semaphore, rf_semaphore)
        };

        let (deferred_set) = {
            let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build();

            let ubo_bindings = [ubo_layout_binding];

            let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_bindings);

            let layout = unsafe { device.create_descriptor_set_layout(&create_info, None)? };

            let layouts = &[layout];

            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(info.uniform_descriptor_pool)
                .set_layouts(layouts);

            let sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

            (sets[0])
        };

        let (deferred_vbo, deferred_vbo_mem) = create_buffer(
            std::mem::size_of::<ColorNormalVertex>() * 100_000,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.memory_props,
            device,
        )?;

        let (deferred_ibo, deferred_ibo_mem) = create_buffer(
            std::mem::size_of::<u32>() * 250_000,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.memory_props,
            device,
        )?;

        let (deferred_ubo, deferred_ubo_mem) = create_buffer(
            std::mem::size_of::<deferred::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.memory_props,
            device,
        )?;

        let vulkan_frame = Self {
            framebuffer: info.framebuffer,
            image_index: info.image_index,
            deferred_set,
            deferred_vbo,
            deferred_vbo_mem,
            deferred_ibo,
            deferred_ibo_mem,
            deferred_ubo,
            image_available_semaphore,
            render_finished_semaphore,
            deferred_ubo_mem,
        };

        Ok(vulkan_frame)
    }
}
