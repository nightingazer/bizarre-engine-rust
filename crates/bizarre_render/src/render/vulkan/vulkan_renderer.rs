use anyhow::anyhow;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension};
use vulkanalia::window as vk_window;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_2::*,
};

use crate::renderer::Renderer;

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
    pub device: VulkanDevices,
    pub surface: vk::SurfaceKHR,
    pub swapchain: VulkanSwapchain,
    pub pipeline: Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,

    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Renderer for VulkanRenderer {
    fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let entry: vulkanalia::Entry;
        let instance: vulkanalia::Instance;
        let device: VulkanDevices;
        let surface: vk::SurfaceKHR;
        let swapchain: VulkanSwapchain;
        let pipeline: Pipeline;
        let framebuffers: Vec<vk::Framebuffer>;

        let debug_messenger: vk::DebugUtilsMessengerEXT;

        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            entry = vulkanalia::Entry::new(loader).map_err(|e| anyhow!(e))?;
            (instance, debug_messenger) = create_instance(window, &entry)?;
            surface = vk_window::create_surface(&instance, &window, &window)?;
            device = VulkanDevices::new(&instance, surface)?;
            swapchain = VulkanSwapchain::new(window, surface, &instance, &device)?;
            pipeline = Pipeline::new(&swapchain, &device.logical)?;
            framebuffers = create_framebuffers(
                &swapchain.image_views,
                &swapchain.extent,
                pipeline.render_pass,
                &device.logical,
            )?;
        }

        #[cfg(debug_assertions)]
        {
            let context = VulkanRenderContext {
                entry,
                instance,
                device,
                surface,
                swapchain,
                pipeline,
                framebuffers,
                debug_messenger,
            };
            Ok(Self { context })
        }

        #[cfg(not(debug_assertions))]
        {
            let context = VulkanRenderContext {
                entry,
                instance,
                device,
                surface,
                swapchain,
                pipeline,
                framebuffers,
            };
            Ok(Self { context })
        }
    }

    fn destroy(&self) -> anyhow::Result<()> {
        unsafe {
            let device = &self.context.device.logical;

            for framebuffer in self.context.framebuffers.iter() {
                device.destroy_framebuffer(*framebuffer, None);
            }

            self.context.pipeline.destroy(device);

            self.context.swapchain.destroy(device);

            self.context.device.destroy();

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
        Ok(())
    }
}
