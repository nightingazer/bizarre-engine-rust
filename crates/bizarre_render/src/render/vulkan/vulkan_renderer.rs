use anyhow::anyhow;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension};
use vulkanalia::window as vk_window;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_2::*,
};

use crate::renderer::Renderer;

use super::pipeline::Pipeline;
use super::{devices::VulkanDevice, instance::create_instance, swapchain::VulkanSwapchain};

#[derive(Debug)]
pub struct VulkanRenderer {
    context: VulkanRenderContext,
    pipeline: Pipeline,
}

#[derive(Debug)]
#[derive(Default)]
pub struct VulkanRenderContext {
    pub entry: Option<vulkanalia::Entry>,
    pub instance: Option<vulkanalia::Instance>,
    pub device: Option<VulkanDevice>,
    pub surface: Option<vk::SurfaceKHR>,
    pub swapchain: Option<VulkanSwapchain>,

    #[cfg(debug_assertions)]
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanRenderContext {
    pub fn entry(mut self, entry: vulkanalia::Entry) -> Self {
        self.entry = Some(entry);
        self
    }

    pub fn instance(mut self, instance: vulkanalia::Instance) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn device(mut self, device: VulkanDevice) -> Self {
        self.device = Some(device);
        self
    }

    pub fn surface(mut self, surface: vk::SurfaceKHR) -> Self {
        self.surface = Some(surface);
        self
    }

    pub fn swapchain(mut self, swapchain: VulkanSwapchain) -> Self {
        self.swapchain = Some(swapchain);
        self
    }

    #[cfg(debug_assertions)]
    pub fn debug_messenger(mut self, debug_messenger: vk::DebugUtilsMessengerEXT) -> Self {
        self.debug_messenger = Some(debug_messenger);
        self
    }
}



impl Renderer for VulkanRenderer {
    fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let entry: vulkanalia::Entry;
        let instance: vulkanalia::Instance;
        let device: VulkanDevice;
        let surface: vk::SurfaceKHR;
        let swapchain: VulkanSwapchain;

        let debug_messenger: vk::DebugUtilsMessengerEXT;

        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            entry = vulkanalia::Entry::new(loader).map_err(|e| anyhow!(e))?;
            (instance, debug_messenger) = create_instance(window, &entry)?;
            surface = vk_window::create_surface(&instance, &window, &window)?;
            device = VulkanDevice::new(&instance, surface)?;
            swapchain = VulkanSwapchain::new(window, surface, &instance, &device)?;
        }

        let mut context = VulkanRenderContext::default()
            .entry(entry)
            .instance(instance)
            .device(device)
            .surface(surface)
            .swapchain(swapchain);

        #[cfg(debug_assertions)]
        {
            context.debug_messenger = Some(debug_messenger);
        }

        let pipeline: Pipeline;

        unsafe {
            pipeline = Pipeline::new(&context)?;
        }

        Ok(Self { context, pipeline })
    }

    fn destroy(&self) -> anyhow::Result<()> {
        unsafe {
            let device = &self.context.device.as_ref().unwrap().logical;

            self.pipeline.destroy(device);

            self.context.swapchain.as_ref().unwrap().destroy(device);

            self.context.device.as_ref().unwrap().destroy();

            #[cfg(debug_assertions)]
            {
                self.context
                    .instance
                    .as_ref()
                    .unwrap()
                    .destroy_debug_utils_messenger_ext(self.context.debug_messenger.unwrap(), None);
            }

            self.context
                .instance
                .as_ref()
                .unwrap()
                .destroy_surface_khr(self.context.surface.unwrap(), None);

            self.context
                .instance
                .as_ref()
                .unwrap()
                .destroy_instance(None);
        }
        Ok(())
    }

    fn render(&self, _window: &winit::window::Window) -> anyhow::Result<()> {
        Ok(())
    }
}
