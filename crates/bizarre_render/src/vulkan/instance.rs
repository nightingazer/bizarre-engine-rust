use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::Result;
use ash::vk;

use crate::vulkan_utils::{debug_utils::create_debug_messenger, instance::create_instance};

pub struct VulkanInstance {
    pub entry: ash::Entry,
    pub instance: ash::Instance,

    #[cfg(feature = "vulkan_debug")]
    _debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanInstance {
    pub unsafe fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let entry = ash::Entry::load()?;
        let instance = create_instance(window, &entry)?;

        #[cfg(feature = "vulkan_debug")]
        let debug_messenger = create_debug_messenger(&entry, &instance)?;

        Ok(Self {
            instance,
            entry,

            #[cfg(feature = "vulkan_debug")]
            _debug_messenger: debug_messenger,
        })
    }
}

impl Deref for VulkanInstance {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for VulkanInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}
