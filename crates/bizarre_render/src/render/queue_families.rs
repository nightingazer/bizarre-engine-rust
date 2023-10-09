use vulkanalia::prelude::v1_2::*;

use crate::errors::SuitabilityError;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: Option<u32>,
}

impl Default for QueueFamilyIndices {
    fn default() -> Self {
        Self { graphics: None }
    }
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

        if let Some(graphuics) = graphuics {
            Ok(Self {
                graphics: Some(graphuics),
            })
        } else {
            Err(SuitabilityError("No suitable queue family found").into())
        }
    }
}
