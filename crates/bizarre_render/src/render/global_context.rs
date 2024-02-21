use std::{
    borrow::Borrow,
    cell::LazyCell,
    sync::{Arc, LazyLock, OnceLock},
};

use anyhow::{anyhow, Result};
use bizarre_logger::{core_critical, core_warn};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event_loop::{EventLoop, EventLoopProxy},
    window::WindowBuilder,
};

use crate::{
    vulkan::{device::VulkanDevice, instance::VulkanInstance},
    vulkan_utils::instance::create_instance,
};

pub struct VulkanContext {
    instance: VulkanInstance,
    device: VulkanDevice,
}

pub struct VulkanGlobalContext {
    context: OnceLock<VulkanContext>,
}

impl VulkanGlobalContext {
    pub const fn new() -> Self {
        Self {
            context: OnceLock::new(),
        }
    }

    pub fn device(&self) -> &VulkanDevice {
        &self
            .context
            .get()
            .expect("Trying to get access to Vulkan Device before the context was initialized!")
            .device
    }

    pub fn instance(&self) -> &VulkanInstance {
        &self
            .context
            .get()
            .expect("Trying to get access to Vulkan Instance before the context was initialized!")
            .instance
    }

    pub fn destroy(&self) {
        let context = self
            .context
            .get()
            .expect("Trying to destroy Vulkan global context before creating one!");
        context.instance.destroy();
        context.device.destroy();
    }
}

pub static VULKAN_GLOBAL_CONTEXT: VulkanGlobalContext = VulkanGlobalContext::new();

pub fn init_vulkan_global_context(window: &winit::window::Window) -> Result<()> {
    #[cfg(debug_assertions)]
    {
        if VULKAN_GLOBAL_CONTEXT.context.get().is_some() {
            core_warn!("Trying to initialize vulkan global context more than once!");
        }
    }

    VULKAN_GLOBAL_CONTEXT.context.set(
        create_global_context(window)
            .map_err(|err| anyhow!("Failed to create a global context for Vulkan: {:?}", err))?,
    );

    Ok(())
}

pub fn destroy_vulkan_global_context() {
    VULKAN_GLOBAL_CONTEXT.destroy();
}

fn create_global_context(window: &winit::window::Window) -> Result<VulkanContext> {
    let instance = VulkanInstance::new(window)?;
    let surface = unsafe {
        ash_window::create_surface(
            &instance.entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )?
    };
    let device = VulkanDevice::new(&instance, surface)?;

    Ok(VulkanContext { instance, device })
}
