use anyhow::anyhow;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::window as vk_window;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_2::*,
};

use crate::renderer::Renderer;

use super::commands::{create_command_buffers, create_command_pool, record_command_buffer};
use super::framebuffer::create_framebuffers;
use super::pipeline::Pipeline;
use super::{devices::VulkanDevices, instance::create_instance, swapchain::VulkanSwapchain};

#[derive(Debug)]
pub struct VulkanRenderer {
    context: VulkanRenderContext,
}

#[derive(Debug)]
pub struct VulkanRenderContext {
    pub entry: vulkanalia::Entry,
    pub instance: vulkanalia::Instance,
    pub devices: VulkanDevices,
    pub surface: vk::SurfaceKHR,
    pub swapchain: VulkanSwapchain,
    pub pipeline: Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,

    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanRenderer {
    unsafe fn render_impl(&self) -> anyhow::Result<()> {
        let device = &self.context.devices.logical;

        let image_index = device
            .acquire_next_image_khr(
                self.context.swapchain.handle,
                u64::MAX,
                self.context.image_available_semaphore,
                vk::Fence::null(),
            )?
            .0 as usize;
        let wait_semaphores = &[self.context.image_available_semaphore];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.context.command_buffers[image_index]];
        let signal_semaphores = &[self.context.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        device.queue_submit(
            self.context.devices.graphics_queue,
            &[submit_info],
            vk::Fence::null(),
        )?;

        let swapchains = &[self.context.swapchain.handle];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        device.queue_present_khr(self.context.devices.present_queue, &present_info)?;
        device.queue_wait_idle(self.context.devices.present_queue)?;

        Ok(())
    }
}

impl Renderer for VulkanRenderer {
    fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let entry: vulkanalia::Entry;
        let instance: vulkanalia::Instance;
        let devices: VulkanDevices;
        let surface: vk::SurfaceKHR;
        let swapchain: VulkanSwapchain;
        let pipeline: Pipeline;
        let framebuffers: Vec<vk::Framebuffer>;
        let command_pool: vk::CommandPool;
        let command_buffers: Vec<vk::CommandBuffer>;
        let image_available_semaphore: vk::Semaphore;
        let render_finished_semaphore: vk::Semaphore;

        let debug_messenger: vk::DebugUtilsMessengerEXT;

        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            entry = vulkanalia::Entry::new(loader).map_err(|e| anyhow!(e))?;
            (instance, debug_messenger) = create_instance(window, &entry)?;
            surface = vk_window::create_surface(&instance, &window, &window)?;
            devices = VulkanDevices::new(&instance, surface)?;
            swapchain = VulkanSwapchain::new(window, surface, &instance, &devices)?;
            pipeline = Pipeline::new(&swapchain, &devices.logical)?;
            framebuffers = create_framebuffers(
                &swapchain.image_views,
                &swapchain.extent,
                pipeline.render_pass,
                &devices.logical,
            )?;
            command_pool = create_command_pool(&instance, &devices, surface)?;
            command_buffers =
                create_command_buffers(framebuffers.len() as u32, command_pool, &devices.logical)?;

            image_available_semaphore = devices
                .logical
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?;
            render_finished_semaphore = devices
                .logical
                .create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?;
        }

        let context;

        #[cfg(debug_assertions)]
        {
            context = VulkanRenderContext {
                entry,
                instance,
                devices,
                surface,
                swapchain,
                pipeline,
                framebuffers,
                command_pool,
                command_buffers,
                render_finished_semaphore,
                image_available_semaphore,

                debug_messenger,
            };
        }

        #[cfg(not(debug_assertions))]
        {
            context = VulkanRenderContext {
                entry,
                instance,
                devices,
                surface,
                swapchain,
                pipeline,
                framebuffers,
                command_pool,
                command_buffers,
                render_finished_semaphore,
                image_available_semaphore,
            };
        }

        for (i, command_buffer) in context.command_buffers.iter().enumerate() {
            unsafe {
                record_command_buffer(command_buffer, i, &context)?;
            }
        }

        Ok(Self { context })
    }

    fn destroy(&self) -> anyhow::Result<()> {
        unsafe {
            let device = &self.context.devices.logical;

            device.device_wait_idle()?;

            device.destroy_semaphore(self.context.image_available_semaphore, None);
            device.destroy_semaphore(self.context.render_finished_semaphore, None);

            device.destroy_command_pool(self.context.command_pool, None);

            for framebuffer in self.context.framebuffers.iter() {
                device.destroy_framebuffer(*framebuffer, None);
            }

            self.context.pipeline.destroy(device);

            self.context.swapchain.destroy(device);

            self.context.devices.destroy();

            #[cfg(debug_assertions)]
            {
                self.context
                    .instance
                    .destroy_debug_utils_messenger_ext(self.context.debug_messenger, None);
            }

            self.context
                .instance
                .destroy_surface_khr(self.context.surface, None);

            self.context.instance.destroy_instance(None);
        }
        Ok(())
    }

    fn render(&self, _window: &winit::window::Window) -> anyhow::Result<()> {
        let result = unsafe {
            self.render_impl()?;
        };

        Ok(result)
    }
}
