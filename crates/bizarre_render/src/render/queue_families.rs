use vulkanalia::prelude::v1_2::*;

use crate::errors::SuitabilityError;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
}

impl QueueFamilyIndices {
    pub unsafe fn new(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> anyhow::Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);
        let graphuics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        if let Some(graphics) = graphuics {
            Ok(Self { graphics })
        } else {
            Err(SuitabilityError("No suitable queue family found").into())
        }
    }
}
