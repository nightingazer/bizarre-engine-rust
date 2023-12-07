use std::sync::Arc;

use anyhow::Result;
use ash::{extensions::khr, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{
    render_package::RenderPackage,
    vulkan::{
        device::VulkanDevice, image::VulkanImage, instance::VulkanInstance,
        swapchain::VulkanSwapchain,
    },
    vulkan_utils::{cmd_buffer::record_submit_cmd_buffer, vulkan_memory::find_memory_type_index},
};

pub struct RenderSystem {
    instance: VulkanInstance,
    device: VulkanDevice,
    surface: vk::SurfaceKHR,
    swapchain: VulkanSwapchain,
    deferred_pipeline: vk::Pipeline,
    uniform_descriptor_pool: vk::DescriptorPool,
}

impl RenderSystem {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = unsafe { VulkanInstance::new(window.clone())? };
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let device = unsafe { VulkanDevice::new(&instance, surface)? };

        let window_extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let (swapchain, image_views) =
            unsafe { VulkanSwapchain::new(&instance, &device, surface, &window_extent)? };

        let uniform_descriptor_pool = unsafe {
            let pool_size = vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(image_views.len() as u32)
                .build();

            let pool_sizes = &[pool_size];

            let pool_info = vk::DescriptorPoolCreateInfo::builder()
                .pool_sizes(pool_sizes)
                .max_sets(image_views.len() as u32);

            device.create_descriptor_pool(&pool_info, None)?
        };

        unsafe {
            let pool_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

            let pool = device.create_command_pool(&pool_create_info, None)?;

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(pool)
                .command_buffer_count(2)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = device.allocate_command_buffers(&command_buffer_allocate_info)?;

            let setup_cmd_buffer = command_buffers[0];
            let draw_cmd_buffer = command_buffers[1];

            let device_memory_properties =
                instance.get_physical_device_memory_properties(device.physical_device);

            let depth_image = VulkanImage::new(
                vk::Extent3D {
                    width: window_extent.width,
                    height: window_extent.height,
                    depth: 1,
                },
                vk::Format::D16_UNORM,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &device_memory_properties,
                &device,
            )?;

            let descriptor_pool = {};

            device.bind_image_memory(*depth_image, depth_image.memory, 0)?;

            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_commands_reuse_fence = device.create_fence(&fence_create_info, None)?;
            let setup_commands_reuse_fence = device.create_fence(&fence_create_info, None)?;

            record_submit_cmd_buffer(
                &device,
                setup_cmd_buffer,
                setup_commands_reuse_fence,
                device.present_queue,
                &[],
                &[],
                &[],
                |device, cmd| {
                    let layout_transition_bariers = vk::ImageMemoryBarrier::builder()
                        .image(*depth_image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1)
                                .build(),
                        )
                        .build();

                    device.cmd_pipeline_barrier(
                        cmd,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_bariers],
                    );
                },
            );

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore =
                device.create_semaphore(&semaphore_create_info, None)?;

            let render_complete_semaphore =
                device.create_semaphore(&semaphore_create_info, None)?;
        }

        let system = Self {
            instance,
            device,
            surface,
            swapchain,
            uniform_descriptor_pool,
            deferred_pipeline: vk::Pipeline::null(),
        };

        Ok(system)
    }

    pub fn render(&mut self, render_package: &RenderPackage) -> Result<()> {
        Ok(())
    }

    pub fn resize(&mut self, size: [u32; 2]) -> Result<()> {
        Ok(())
    }
}

impl Drop for RenderSystem {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(self.uniform_descriptor_pool, None);

            self.device.destroy_device(None);

            #[cfg(feature = "vulkan_debug")]
            self.instance
                .debug_utils_loader
                .destroy_debug_utils_messenger(self.instance.debug_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}
