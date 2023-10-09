use bizarre_logger::{core_info, core_warn};
use thiserror::Error;
use vulkanalia::prelude::v1_2::*;

use crate::{errors::SuitabilityError, queue_families::QueueFamilyIndices};

#[derive(Debug)]
pub struct VulkanDevice {
    pub handle: vk::Device,
    pub physical_device: vk::PhysicalDevice,
    pub queue_family_indices: QueueFamilyIndices,
}

impl VulkanDevice {
    pub unsafe fn new(instance: &Instance) -> anyhow::Result<Self> {
        let physical_device = pick_physical_device(instance)?;
        let queue_family_indices = QueueFamilyIndices::new(instance, physical_device)?;
        Ok(Self {
            handle: Default::default(),
            physical_device,
            queue_family_indices,
        })
    }
}

unsafe fn pick_physical_device(instance: &Instance) -> anyhow::Result<vk::PhysicalDevice> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(err) = check_physical_device(instance, physical_device) {
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
) -> anyhow::Result<()> {
    QueueFamilyIndices::new(instance, physical_device)?;

    let properties = instance.get_physical_device_properties(physical_device);
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(SuitabilityError("Only discrete GPUs are supported").into());
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.geometry_shader != vk::TRUE {
        return Err(SuitabilityError("Device does not support geometry shaders").into());
    }

    Ok(())
}
