use std::ops::{Deref, DerefMut};

use anyhow::Result;
use ash::{extensions::khr, vk};

use super::instance::VulkanInstance;

pub struct VulkanDevice {
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub present_queue: vk::Queue,
    pub queue_family_index: u32,
}

impl VulkanDevice {
    pub unsafe fn new(instance: &VulkanInstance, surface: vk::SurfaceKHR) -> Result<Self> {
        let pdevices = instance
            .enumerate_physical_devices()
            .expect("Failed to find any physical device");

        let surface_loader = khr::Surface::new(&instance.entry, instance);

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

        Ok(Self {
            device,
            physical_device: pdevice,
            present_queue,
            queue_family_index,
        })
    }
}

impl Deref for VulkanDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl DerefMut for VulkanDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.device
    }
}
