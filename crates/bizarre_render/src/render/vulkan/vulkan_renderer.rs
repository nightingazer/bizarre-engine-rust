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

const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Debug)]
pub struct VulkanRenderer {
    context: VulkanRenderContext,
    resized: bool,
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
    pub image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub frame: usize,
    pub in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
    pub images_in_flight: Vec<vk::Fence>,
    pub window_height: u32,
    pub window_width: u32,

    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanRenderer {
    unsafe fn on_resize_impl(&mut self) -> anyhow::Result<()> {
        self.recreate_swapchain()?;

        self.context.pipeline =
            Pipeline::new(&self.context.swapchain, &self.context.devices.logical)?;
        self.context.framebuffers = create_framebuffers(
            &self.context.swapchain.image_views,
            &self.context.swapchain.extent,
            self.context.pipeline.render_pass,
            &self.context.devices.logical,
        )?;
        self.context.command_buffers = create_command_buffers(
            self.context.framebuffers.len() as u32,
            self.context.command_pool,
            &self.context.devices.logical,
        )?;
        self.context
            .images_in_flight
            .resize(self.context.framebuffers.len(), vk::Fence::null());

        Ok(())
    }

    unsafe fn recreate_swapchain(&mut self) -> anyhow::Result<()> {
        let device = &self.context.devices.logical;

        device.device_wait_idle()?;
        let window_size = (self.context.window_width, self.context.window_height);

        let new_swapchain = VulkanSwapchain::new(
            window_size,
            self.context.surface,
            &self.context.instance,
            &self.context.devices,
        )?;

        self.destroy_swapchain()?;

        self.context.swapchain = new_swapchain;

        Ok(())
    }

    unsafe fn destroy_swapchain(&self) -> anyhow::Result<()> {
        let device = &self.context.devices.logical;

        for framebuffer in self.context.framebuffers.iter() {
            device.destroy_framebuffer(*framebuffer, None);
        }

        device.free_command_buffers(
            self.context.command_pool,
            self.context.command_buffers.as_slice(),
        );
        self.context.pipeline.destroy(device);
        self.context.swapchain.destroy(device);

        Ok(())
    }

    unsafe fn render_impl(&mut self) -> anyhow::Result<()> {
        let device = &self.context.devices.logical;

        device.wait_for_fences(
            &[self.context.in_flight_fences[self.context.frame]],
            true,
            u64::MAX,
        )?;

        let result = device.acquire_next_image_khr(
            self.context.swapchain.handle,
            u64::MAX,
            self.context.image_available_semaphores[self.context.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => {
                return self.on_resize((self.context.window_width, self.context.window_height));
            }
            Err(e) => return Err(anyhow!(e)),
        };

        if !self.context.images_in_flight[image_index].is_null() {
            device.wait_for_fences(
                &[self.context.images_in_flight[image_index]],
                true,
                u64::MAX,
            )?;
        }

        self.context.images_in_flight[image_index] =
            self.context.in_flight_fences[self.context.frame];

        let wait_semaphores = &[self.context.image_available_semaphores[self.context.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.context.command_buffers[image_index]];
        let signal_semaphores = &[self.context.render_finished_semaphores[self.context.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        device.reset_fences(&[self.context.in_flight_fences[self.context.frame]])?;

        device.queue_submit(
            self.context.devices.graphics_queue,
            &[submit_info],
            self.context.in_flight_fences[self.context.frame],
        )?;

        let swapchains = &[self.context.swapchain.handle];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        device.queue_present_khr(self.context.devices.present_queue, &present_info)?;

        self.context.frame = (self.context.frame + 1) % MAX_FRAMES_IN_FLIGHT;

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
        let mut image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT] = Default::default();

        let mut images_in_flight: Vec<vk::Fence> = vec![];

        let debug_messenger: vk::DebugUtilsMessengerEXT;

        let window_size = window.inner_size();
        let window_size = (window_size.width, window_size.height);

        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            entry = vulkanalia::Entry::new(loader).map_err(|e| anyhow!(e))?;
            (instance, debug_messenger) = create_instance(window, &entry)?;
            surface = vk_window::create_surface(&instance, &window, &window)?;
            devices = VulkanDevices::new(&instance, surface)?;
            swapchain = VulkanSwapchain::new(window_size, surface, &instance, &devices)?;
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

            let semaphore_info = vk::SemaphoreCreateInfo::builder();
            let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                image_available_semaphores[i] =
                    devices.logical.create_semaphore(&semaphore_info, None)?;
                render_finished_semaphores[i] =
                    devices.logical.create_semaphore(&semaphore_info, None)?;

                in_flight_fences[i] = devices.logical.create_fence(&fence_info, None)?;
            }

            images_in_flight = swapchain.images.iter().map(|_| vk::Fence::null()).collect();
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
                render_finished_semaphores,
                image_available_semaphores,
                frame: 0,
                in_flight_fences,
                images_in_flight,
                window_width: window_size.0,
                window_height: window_size.1,

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
                frame: 0,
                in_flight_fences,
                images_in_flight,
                window_width: window_size.0,
                window_height: window_size.1,
            };
        }

        for (i, command_buffer) in context.command_buffers.iter().enumerate() {
            unsafe {
                record_command_buffer(command_buffer, i, &context)?;
            }
        }

        Ok(Self {
            context,
            resized: false,
        })
    }

    fn on_resize(&mut self, window_size: (u32, u32)) -> anyhow::Result<()> {
        self.resized = true;
        self.context.window_width = window_size.0;
        self.context.window_height = window_size.1;

        Ok(())
    }

    fn destroy(&mut self) -> anyhow::Result<()> {
        unsafe {
            let device = &self.context.devices.logical;

            device.device_wait_idle()?;

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.destroy_semaphore(self.context.image_available_semaphores[i], None);
                device.destroy_semaphore(self.context.render_finished_semaphores[i], None);
                device.destroy_fence(self.context.in_flight_fences[i], None);
            }

            self.destroy_swapchain();

            device.destroy_command_pool(self.context.command_pool, None);

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

    fn render(&mut self, _window: &winit::window::Window) -> anyhow::Result<()> {
        let result = unsafe {
            if (self.resized) {
                self.on_resize_impl()?;
                self.resized = false;
            }

            self.render_impl()?;
        };

        Ok(result)
    }
}
