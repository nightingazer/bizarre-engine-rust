use anyhow::anyhow;
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    vk::{self, ExtDebugUtilsExtension},
};

use crate::{devices::VulkanDevice, instance::create_instance};
use vulkanalia::prelude::v1_2::*;

#[derive(Debug)]
pub struct Renderer {
    entry: vulkanalia::Entry,
    instance: vulkanalia::Instance,
    device: VulkanDevice,

    #[cfg(debug_assertions)]
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let entry: vulkanalia::Entry;
        let instance: vulkanalia::Instance;
        let device: VulkanDevice;

        #[cfg(debug_assertions)]
        let debug_messenger: vk::DebugUtilsMessengerEXT;

        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            entry = vulkanalia::Entry::new(loader).map_err(|e| anyhow!(e))?;
            (instance, debug_messenger) = create_instance(window, &entry)?;
            device = VulkanDevice::new(&instance)?;
        }

        #[cfg(debug_assertions)]
        return Ok(Self {
            entry,
            instance,
            device,
            debug_messenger,
        });

        #[cfg(not(debug_assertions))]
        return Ok(Self {
            entry,
            instance,
            device,
            physical_device,
        });
    }

    pub fn destroy(&mut self) -> anyhow::Result<()> {
        unsafe {
            self.device.destroy();

            #[cfg(debug_assertions)]
            self.instance
                .destroy_debug_utils_messenger_ext(self.debug_messenger, None);

            self.instance.destroy_instance(None);
        }
        Ok(())
    }

    pub fn render(&self, window: &winit::window::Window) -> anyhow::Result<()> {
        Ok(())
    }
}
