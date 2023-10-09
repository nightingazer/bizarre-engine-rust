use anyhow::anyhow;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    vk::{self, ExtDebugUtilsExtension, KhrSwapchainExtension},
    window as vk_window,
};

use vulkanalia::vk::KhrSurfaceExtension;

use crate::{devices::VulkanDevice, instance::create_instance, swapchain::VulkanSwapchain};
use vulkanalia::prelude::v1_2::*;

#[derive(Debug)]
pub struct Renderer {
    entry: vulkanalia::Entry,
    instance: vulkanalia::Instance,
    device: VulkanDevice,
    surface: vk::SurfaceKHR,
    swapchain: VulkanSwapchain,

    #[cfg(debug_assertions)]
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
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
            swapchain = VulkanSwapchain::new(&window, surface, &instance, &device)?;
        }

        #[cfg(debug_assertions)]
        return Ok(Self {
            entry,
            instance,
            device,
            surface,
            swapchain,

            debug_messenger,
        });

        #[cfg(not(debug_assertions))]
        return Ok(Self {
            entry,
            instance,
            device,
            surface,
            swapchain,
        });
    }

    pub fn destroy(&mut self) -> anyhow::Result<()> {
        unsafe {
            self.device
                .logical
                .destroy_swapchain_khr(self.swapchain.handle, None);

            self.device.destroy();

            #[cfg(debug_assertions)]
            self.instance
                .destroy_debug_utils_messenger_ext(self.debug_messenger, None);

            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
        Ok(())
    }

    pub fn render(&self, window: &winit::window::Window) -> anyhow::Result<()> {
        Ok(())
    }
}
