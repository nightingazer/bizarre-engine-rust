use std::sync::Arc;

use anyhow::Result;
use ash::{extensions::khr, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{render_package::RenderPackage, vulkan::instance::VulkanInstance};

pub struct RenderSystem {
    instance: VulkanInstance,
}

impl RenderSystem {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = unsafe { VulkanInstance::new(window.clone())? };

        unsafe {
            let pdevices = instance
                .enumerate_physical_devices()
                .expect("Failed to find any physical device");

            let surface = ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?;

            let surface_loader = khr::Surface::new(&instance.entry, &instance);

            let (pdevice, queue_family_index) = pdevices
                .iter()
                .find_map(|p| {
                    instance
                        .get_physical_device_queue_family_properties(*p)
                        .iter()
                        .enumerate()
                        .find_map(|(i, q)| {
                            let supports_graphics_and_surface =
                                q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(*p, i as u32, surface)
                                        .unwrap();
                            if supports_graphics_and_surface {
                                Some((*p, i))
                            } else {
                                None
                            }
                        })
                })
                .expect("Failed to find suitable device.");

            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
            let pdevice_features = vk::PhysicalDeviceFeatures::builder()
                .shader_clip_distance(true)
                .build();
            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&pdevice_features);

            let device = instance
                .create_device(pdevice, &device_create_info, None)
                .expect("Failed to create a Vulkan Device");

            let present_queue = device.get_device_queue(queue_family_index, 0);

            let surface_format = surface_loader
                .get_physical_device_surface_formats(pdevice, surface)
                .expect("Failed to get surface formats")[0];
        }

        let system = Self { instance };

        Ok(system)
    }

    pub fn render(&mut self, render_package: &RenderPackage) -> Result<()> {
        Ok(())
    }

    pub fn resize(&mut self, size: [u32; 2]) -> Result<()> {
        Ok(())
    }
}
