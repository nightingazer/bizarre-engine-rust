use crate::vulkan::swapchain::SwapchainSupport;
use std::collections::HashSet;

use bizarre_logger::{core_info, core_warn};
use vulkanalia::prelude::v1_2::*;

use crate::vulkan::vulkan_constants::VALIDATION_LAYER;

use super::{
    queue_families::QueueFamilyIndices, vulkan_constants::DEVICE_EXTENSIONS,
    vulkan_errors::SuitabilityError,
};

#[derive(Debug)]
pub struct VulkanDevice {
    pub logical: Device,
    pub physical: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl VulkanDevice {
    pub unsafe fn new(instance: &Instance, surface: vk::SurfaceKHR) -> anyhow::Result<Self> {
        let physical = pick_physical_device(instance, surface)?;
        let queue_family_indices = QueueFamilyIndices::new(instance, physical, surface)?;
        let logical = create_logical_device(instance, physical, &queue_family_indices)?;
        let graphics_queue = logical.get_device_queue(queue_family_indices.graphics, 0);
        let present_queue = logical.get_device_queue(queue_family_indices.present, 0);

        Ok(Self {
            logical,
            physical,
            graphics_queue,
            present_queue,
        })
    }

    pub unsafe fn destroy(&self) {
        self.logical.destroy_device(None);
    }
}

unsafe fn pick_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<vk::PhysicalDevice> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(err) = check_physical_device(instance, physical_device, surface) {
            core_warn!(
                "Skipping physical device ('{}'): {}",
                properties.device_name,
                err
            );
        } else {
            core_info!("Selected physical device: '{}'", properties.device_name);
            return Ok(physical_device);
        }
    }

    Err(SuitabilityError("No suitable physical device found").into())
}

unsafe fn check_physical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
) -> anyhow::Result<()> {
    QueueFamilyIndices::new(instance, physical_device, surface)?;

    check_physical_device_extensions(instance, physical_device)?;

    let properties = instance.get_physical_device_properties(physical_device);
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(SuitabilityError("Only discrete GPUs are supported").into());
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.geometry_shader != vk::TRUE {
        return Err(SuitabilityError("Device does not support geometry shaders").into());
    }

    let support = SwapchainSupport::get(instance, physical_device, surface)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        Err(SuitabilityError("Insufficient swapchain support").into())
    } else {
        Ok(())
    }
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> anyhow::Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(SuitabilityError("Missing required device extensions").into())
    }
}

unsafe fn create_logical_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    indices: &QueueFamilyIndices,
) -> anyhow::Result<Device> {
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(indices.graphics)
        .queue_priorities(queue_priorities);

    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    let layers = if cfg!(debug_assertions) {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let features = vk::PhysicalDeviceFeatures::builder();

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(physical_device, &info, None)?;

    Ok(device)
}
