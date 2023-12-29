use anyhow::Result;
use ash::vk;

use crate::vulkan_utils::framebuffer::create_framebuffer;

pub struct VulkanFrame {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub render_cmd: vk::CommandBuffer,
    pub render_cmd_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
}

pub struct VulkanFrameInfo {
    pub present_image: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub image_index: u32,
    pub render_cmd: vk::CommandBuffer,
}

impl VulkanFrame {
    pub fn new(info: &VulkanFrameInfo, device: &ash::Device) -> Result<Self> {
        let (image_available_semaphore, render_finished_semaphore) = unsafe {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let ia_semaphore = device.create_semaphore(&semaphore_create_info, None)?;
            let rf_semaphore = device.create_semaphore(&semaphore_create_info, None)?;

            (ia_semaphore, rf_semaphore)
        };

        let framebuffer_attachments = [info.present_image];
        let framebuffer = create_framebuffer(
            &framebuffer_attachments,
            info.extent,
            info.render_pass,
            device,
        )?;

        let extent = vk::Extent3D {
            width: info.extent.width,
            height: info.extent.height,
            depth: 1,
        };

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let render_cmd_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let vulkan_frame = Self {
            image_index: info.image_index,
            framebuffer,
            render_cmd: info.render_cmd,
            render_cmd_fence,
            image_available_semaphore,
            render_finished_semaphore,
        };

        Ok(vulkan_frame)
    }

    pub fn destroy(&mut self, cmd_pool: vk::CommandPool, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            self.image_available_semaphore = vk::Semaphore::null();

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            device.destroy_framebuffer(self.framebuffer, None);
            self.framebuffer = vk::Framebuffer::null();

            device.destroy_fence(self.render_cmd_fence, None);
            self.render_cmd_fence = vk::Fence::null();

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            let cmd_bufs = [self.render_cmd];
            device.free_command_buffers(cmd_pool, &cmd_bufs);
            self.render_cmd = vk::CommandBuffer::null();
        }
    }
}
