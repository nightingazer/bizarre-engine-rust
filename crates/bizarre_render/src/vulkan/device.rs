use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use anyhow::{bail, Result};
use ash::{extensions::khr, vk};
use bizarre_logger::core_info;

use super::instance::VulkanInstance;

pub struct VulkanDevice {
    pub handle: ash::Device,
    pub physical_device: vk::PhysicalDevice,
    pub memory_props: vk::PhysicalDeviceMemoryProperties,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_family_index: u32,
}

impl VulkanDevice {
    pub fn new(instance: &VulkanInstance, surface: vk::SurfaceKHR) -> Result<Self> {
        let pdevices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to find any physical device")
        };

        let surface_loader = khr::Surface::new(&instance.entry, instance);

        let pdevice_props = {
            pdevices
                .iter()
                .map(|p| unsafe { instance.get_physical_device_properties(*p) })
                .collect::<Vec<_>>()
        };

        let pdevice_names = pdevice_props
            .iter()
            .map(|p| {
                let name = unsafe { CStr::from_ptr(p.device_name.as_ptr()).to_str()?.to_string() };
                Ok(name)
            })
            .collect::<Result<Vec<_>>>()?;

        let message = pdevice_names
            .iter()
            .fold("".to_string(), |acc, n| format!("{}\n\t{}", acc, n));
        core_info!("Available GPUs:{}", message);

        let mut pdevices_map = pdevices
            .iter()
            .zip(pdevice_names)
            .zip(pdevice_props)
            .map(|((p, n), props)| unsafe {
                let queue_index = instance
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
                            Some(i)
                        } else {
                            None
                        }
                    });
                if queue_index.is_none() {
                    return (*p, 0, 0, n);
                }

                let mut rating = 0;

                let memory_props = instance.get_physical_device_memory_properties(*p);

                rating += memory_props
                    .memory_heaps
                    .iter()
                    .map(|m| m.size)
                    .reduce(|acc, e| acc + e)
                    .unwrap_or(0);

                rating *= match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
                    _ => 1,
                };

                (*p, rating, queue_index.unwrap(), n)
            })
            .collect::<Vec<_>>();

        pdevices_map.sort_by(|a, b| a.1.cmp(&b.1).reverse());

        if pdevices_map.first().is_none() || pdevices_map.first().unwrap().1 == 0 {
            bail!("Failed to find suitable physical device");
        }

        let (pdevice, _, queue_family_index, pdevice_name) = pdevices_map.first().unwrap();

        core_info!("Selected GPU: {}", pdevice_name);

        let queue_family_index = *queue_family_index as u32;
        let device_extension_names_raw = [khr::Swapchain::name().as_ptr()];
        let pdevice_features = vk::PhysicalDeviceFeatures::builder()
            .shader_clip_distance(true)
            .sample_rate_shading(true)
            .build();
        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities);

        let mut synchronization_feature = vk::PhysicalDeviceSynchronization2Features::builder()
            .synchronization2(true)
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&pdevice_features)
            .push_next(&mut synchronization_feature);

        let device = unsafe {
            instance
                .create_device(*pdevice, &device_create_info, None)
                .expect("Failed to create a Vulkan Device")
        };

        let (graphics_queue, present_queue) = unsafe {
            let graphics_queue = device.get_device_queue(queue_family_index, 0);
            let present_queue = device.get_device_queue(queue_family_index, 0);
            (graphics_queue, present_queue)
        };

        let memory_props = unsafe { instance.get_physical_device_memory_properties(*pdevice) };

        Ok(Self {
            handle: device,
            physical_device: *pdevice,
            memory_props,
            graphics_queue,
            present_queue,
            queue_family_index,
        })
    }

    pub fn find_memory_type_index(
        &self,
        memory_req: &vk::MemoryRequirements,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        self.memory_props.memory_types[..self.memory_props.memory_type_count as _]
            .iter()
            .enumerate()
            .find(|(i, memory_type)| {
                (1 << i) & memory_req.memory_type_bits != 0
                    && memory_type.property_flags & flags == flags
            })
            .map(|(i, _)| i as u32)
    }

    pub fn destroy(&self) {
        unsafe {
            self.destroy_device(None);
        }
    }
}

impl Deref for VulkanDevice {
    type Target = ash::Device;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for VulkanDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
