use std::{
    borrow::Borrow,
    cell::LazyCell,
    sync::{Arc, LazyLock, OnceLock},
};

use anyhow::{anyhow, Result};
use ash::vk;
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

pub static VULKAN_GLOBAL_CONTEXT: VulkanGlobalContext = VulkanGlobalContext::new();

pub struct VulkanContext {
    instance: VulkanInstance,
    device: VulkanDevice,
    physical_memory_properties: vk::PhysicalDeviceMemoryProperties,
    descriptor_pool: vk::DescriptorPool,
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

    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self
            .context
            .get()
            .expect("Trying to get access to Vulkan global context before it was initialized!")
            .physical_memory_properties
    }

    pub fn descriptor_pool(&self) -> vk::DescriptorPool {
        self.context
            .get()
            .expect("Trying to get access to Vulkan global context before it was initialized!")
            .descriptor_pool
    }

    pub fn destroy(&self) {
        let context = self
            .context
            .get()
            .expect("Trying to destroy Vulkan global context before creating one!");

        unsafe {
            context
                .device
                .destroy_descriptor_pool(context.descriptor_pool, None);
        }

        context.device.destroy();
        context.instance.destroy();
    }
}

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

    let physical_memory_properties =
        unsafe { instance.get_physical_device_memory_properties(device.physical_device) };

    let descriptor_pool = {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(10)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::INPUT_ATTACHMENT)
                .descriptor_count(10)
                .build(),
        ];
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(512)
            .pool_sizes(&pool_sizes);

        unsafe { device.create_descriptor_pool(&create_info, None)? }
    };

    Ok(VulkanContext {
        instance,
        device,
        physical_memory_properties,
        descriptor_pool,
    })
}
