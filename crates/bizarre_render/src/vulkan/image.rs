use std::ops::{Deref, DerefMut};

use anyhow::Result;
use ash::vk;

use crate::{
    global_context::VULKAN_GLOBAL_CONTEXT, vulkan_utils::vulkan_memory::find_memory_type_index,
};

pub struct VulkanImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    pub format: vk::Format,
    pub extent: vk::Extent3D,
}

impl VulkanImage {
    pub fn new(
        extent: vk::Extent3D,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        usage: vk::ImageUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let device = VULKAN_GLOBAL_CONTEXT.device();

        let image = {
            let image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(format)
                .extent(extent)
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED);

            unsafe { device.create_image(&image_create_info, None)? }
        };

        let memory_requirements = unsafe { device.get_image_memory_requirements(image) };

        let memory_type_index = find_memory_type_index(&memory_requirements, memory_flags).ok_or(
            anyhow::anyhow!("Failed to find suitable memory type for image allocation"),
        )?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .memory_type_index(memory_type_index)
            .allocation_size(memory_requirements.size);

        let memory = unsafe { device.allocate_memory(&allocate_info, None)? };

        unsafe { device.bind_image_memory(image, memory, 0)? };

        let view = {
            let view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(aspect_flags)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                );

            unsafe { device.create_image_view(&view_create_info, None)? }
        };

        Ok(Self {
            image,
            view,
            memory,
            format,
            extent,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.memory, None);

            self.view = vk::ImageView::null();
            self.image = vk::Image::null();
            self.memory = vk::DeviceMemory::null();
        }
    }
}

impl Deref for VulkanImage {
    type Target = vk::Image;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl DerefMut for VulkanImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image
    }
}
