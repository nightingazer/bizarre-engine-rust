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
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    #[cfg(feature = "vulkan_debug")]
    pub debug_utils_loader: ash::extensions::ext::DebugUtils,
}

impl VulkanInstance {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        let (entry, instance) = unsafe {
            let entry = ash::Entry::load()?;
            let instance = create_instance(window, &entry)?;
            (entry, instance)
        };

        #[cfg(feature = "vulkan_debug")]
        let (debug_messenger, debug_utils_loader) =
            unsafe { create_debug_messenger(&entry, &instance)? };

        Ok(Self {
            instance,
            entry,

            #[cfg(feature = "vulkan_debug")]
            debug_messenger,
            #[cfg(feature = "vulkan_debug")]
            debug_utils_loader,
        })
    }

    pub fn destroy(&self) {
        #[cfg(feature = "vulkan_debug")]
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
        }

        unsafe {
            self.instance.destroy_instance(None);
        }
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
