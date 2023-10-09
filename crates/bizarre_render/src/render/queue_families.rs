use vulkanalia::{prelude::v1_2::*, vk::KhrSurfaceExtension};

use crate::errors::SuitabilityError;

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub present: u32,
}

impl QueueFamilyIndices {
    pub unsafe fn new(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<Self> {
        let properties = instance.get_physical_device_queue_family_properties(physical_device);

        let mut graphics = None;
        let mut present = None;

        for (index, properties) in properties.iter().enumerate() {
            if instance.get_physical_device_surface_support_khr(
                physical_device,
                index as u32,
                surface,
            )? {
                present = Some(index as u32);
            }

            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(index as u32);
            }

            if let (Some(_), Some(_)) = (graphics, present) {
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        } else {
            Err(SuitabilityError("Missing some queue families").into())
        }
    }
}
