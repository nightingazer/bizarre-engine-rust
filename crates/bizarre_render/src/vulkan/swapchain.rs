use std::ops::{Deref, DerefMut};

use anyhow::{bail, Result};
use ash::{extensions::khr, vk};

use crate::global_context::VULKAN_GLOBAL_CONTEXT;

use super::{device::VulkanDevice, instance::VulkanInstance};

pub struct VulkanSwapchain {
    pub handle: vk::SwapchainKHR,
    pub image_format: vk::Format,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub swapchain_loader: khr::Swapchain,
    pub surface_loader: khr::Surface,
    pub image_views: Vec<vk::ImageView>,
}

impl Deref for VulkanSwapchain {
    type Target = vk::SwapchainKHR;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for VulkanSwapchain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl VulkanSwapchain {
    pub fn new(
        instance: &VulkanInstance,
        device: &VulkanDevice,
        surface: vk::SurfaceKHR,
        extent: &vk::Extent2D,
    ) -> Result<Self> {
        let surface_loader = khr::Surface::new(&instance.entry, instance);

        let (surface_format, surface_capabilities, present_modes) = unsafe {
            let surface_format = surface_loader
                .get_physical_device_surface_formats(device.physical_device, surface)
                .expect("Failed to get surface formats")[0];

            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities(device.physical_device, surface)
                .expect("Failed to get surface capabilities");

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(device.physical_device, surface)
                .unwrap();

            (surface_format, surface_capabilities, present_modes)
        };

        let swapchain_loader = khr::Swapchain::new(instance, device);

        let swapchain = create_swapchain(
            surface_capabilities,
            extent,
            surface,
            surface_format,
            &present_modes,
            &swapchain_loader,
        )?;

        let image_views = create_image_views(&swapchain_loader, swapchain, surface_format, device)?;

        Ok(Self {
            handle: swapchain,
            surface_capabilities,
            swapchain_loader,
            surface_loader,
            surface_format,
            image_format: surface_format.format,
            image_views,
        })
    }

    pub fn recreate(&mut self, extent: &vk::Extent2D, surface: vk::SurfaceKHR) -> Result<()> {
        self.destroy();

        let device = VULKAN_GLOBAL_CONTEXT.device();

        let (surface_capabilities, present_modes) = unsafe {
            let surface_capabilities = self
                .surface_loader
                .get_physical_device_surface_capabilities(device.physical_device, surface)?;

            let present_modes = self
                .surface_loader
                .get_physical_device_surface_present_modes(device.physical_device, surface)?;

            (surface_capabilities, present_modes)
        };

        self.handle = create_swapchain(
            surface_capabilities,
            &surface_capabilities.current_extent,
            surface,
            self.surface_format,
            &present_modes,
            &self.swapchain_loader,
        )?;

        self.image_views = create_image_views(
            &self.swapchain_loader,
            self.handle,
            self.surface_format,
            device,
        )?;

        Ok(())
    }

    pub fn get_image_views(&self) -> Result<Vec<vk::ImageView>> {
        Ok(self.image_views.clone())
    }

    pub fn destroy(&mut self) -> Result<()> {
        #[cfg(debug_assertions)]
        {
            if self.handle == vk::SwapchainKHR::null() {
                bail!("Trying to destroy a swapchain with null handle!");
            }
        }

        unsafe {
            self.swapchain_loader.destroy_swapchain(self.handle, None);
            self.handle = vk::SwapchainKHR::null();
        }

        #[cfg(debug_assertions)]
        {
            if self.image_views.iter().any(|i| *i == vk::ImageView::null()) {
                bail!("Trying to destroy swapchain image views but some of them are null!")
            }
        }

        let device = VULKAN_GLOBAL_CONTEXT.device();

        unsafe {
            self.image_views.iter_mut().for_each(|image| {
                device.destroy_image_view(*image, None);
                *image = vk::ImageView::null();
            });
        }

        Ok(())
    }
}

fn create_swapchain(
    surface_capabilities: vk::SurfaceCapabilitiesKHR,
    extent: &vk::Extent2D,
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,
    present_modes: &Vec<vk::PresentModeKHR>,
    swapchain_loader: &khr::Swapchain,
) -> Result<vk::SwapchainKHR> {
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

    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
    Ok(swapchain)
}

#[inline]
fn create_image_views(
    swapchain_loader: &khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    surface_format: vk::SurfaceFormatKHR,
    device: &VulkanDevice,
) -> Result<Vec<vk::ImageView>> {
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
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
            let view = unsafe { device.create_image_view(&create_view_info, None)? };
            Ok(view)
        })
        .collect::<Result<Vec<_>>>()
        .expect("Failed to create image views");
    Ok(image_views)
}
