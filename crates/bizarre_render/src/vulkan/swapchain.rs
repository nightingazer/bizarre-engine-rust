use anyhow::Result;
use ash::{extensions::khr, vk};

use super::{device::VulkanDevice, instance::VulkanInstance};

pub struct VulkanSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub swapchain_loader: khr::Swapchain,
}

impl VulkanSwapchain {
    pub unsafe fn new(
        instance: &VulkanInstance,
        device: &VulkanDevice,
        surface: vk::SurfaceKHR,
        extent: &vk::Extent2D,
    ) -> Result<(Self, Vec<vk::ImageView>)> {
        let surface_loader = khr::Surface::new(&instance.entry, instance);

        let surface_format = surface_loader
            .get_physical_device_surface_formats(device.physical_device, surface)
            .expect("Failed to get surface formats")[0];

        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities(device.physical_device, surface)
            .expect("Failed to get surface capabilities");

        let mut desired_image_count = surface_capabilities.min_image_count + 1;
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }

        let surface_resolution = match surface_capabilities.current_extent.width {
            u32::MAX => *extent,
            _ => surface_capabilities.current_extent,
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR)
        {
            vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR
        } else {
            surface_capabilities.current_transform
        };

        let present_modes = surface_loader
            .get_physical_device_surface_present_modes(device.physical_device, surface)
            .unwrap();

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain_loader = khr::Swapchain::new(instance, device);
        let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;

        let images = swapchain_loader.get_swapchain_images(swapchain)?;
        let image_views = images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    )
                    .image(image);
                let view = device.create_image_view(&create_view_info, None)?;
                Ok(view)
            })
            .collect::<Result<Vec<_>>>()
            .expect("Failed to create image views");

        Ok((
            Self {
                swapchain,
                surface_capabilities,
                swapchain_loader,
            },
            image_views,
        ))
    }
}
