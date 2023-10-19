use bizarre_logger::core_warn;
use vulkanalia::prelude::v1_2::*;
use vulkanalia::vk::{self, KhrSurfaceExtension, KhrSwapchainExtension};

use super::devices::VulkanDevices;
use super::queue_families::QueueFamilyIndices;
use super::vulkan_renderer::VulkanRenderContext;

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub unsafe fn get(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            capabilities: instance
                .get_physical_device_surface_capabilities_khr(physical_device, surface)?,
            formats: instance.get_physical_device_surface_formats_khr(physical_device, surface)?,
            present_modes: instance
                .get_physical_device_surface_present_modes_khr(physical_device, surface)?,
        })
    }
}

#[derive(Debug)]
pub struct VulkanSwapchain {
    pub handle: vk::SwapchainKHR,
    pub format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl VulkanSwapchain {
    pub unsafe fn new(
        window_size: (u32, u32),
        surface: vk::SurfaceKHR,
        instance: &Instance,
        vulkan_devices: &VulkanDevices,
    ) -> anyhow::Result<Self> {
        let support = SwapchainSupport::get(instance, vulkan_devices.physical, surface)?;

        let format = get_swapchain_surface_format(&support.formats);
        let extent = get_swapchain_extent(window_size, support.capabilities);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            core_warn!(
                "Image count ({}) exceeds maximum ({}). Using maximum instead.",
                image_count,
                support.capabilities.max_image_count
            );
            image_count = support.capabilities.max_image_count;
        }

        let handle = create_swapchain(
            surface,
            image_count,
            &format,
            &support,
            extent,
            vulkan_devices,
            instance,
        )?;

        let images = vulkan_devices.logical.get_swapchain_images_khr(handle)?;
        let image_views = create_image_views(&vulkan_devices.logical, format.format, &images)?;

        Ok(Self {
            handle,
            format,
            extent,
            images,
            image_views,
        })
    }

    pub unsafe fn recreate_image_views(&mut self, device: &Device) -> anyhow::Result<()> {
        self.image_views = create_image_views(device, self.format.format, &self.images)?;

        Ok(())
    }

    pub unsafe fn destroy(&self, device: &Device) {
        self.image_views.iter().for_each(|i| {
            device.destroy_image_view(*i, None);
        });

        device.destroy_swapchain_khr(self.handle, None);
    }
}

unsafe fn create_swapchain(
    surface: vk::SurfaceKHR,
    image_count: u32,
    format: &vk::SurfaceFormatKHR,
    support: &SwapchainSupport,
    extent: vk::Extent2D,
    devices: &VulkanDevices,
    instance: &Instance,
) -> anyhow::Result<vk::SwapchainKHR> {
    let indices = QueueFamilyIndices::get(instance, devices.physical, surface)?;
    let mut queue_family_indices = vec![];
    let image_sharing_mode = if indices.graphics != indices.present {
        queue_family_indices.push(indices.graphics);
        queue_family_indices.push(indices.present);
        vk::SharingMode::CONCURRENT
    } else {
        vk::SharingMode::EXCLUSIVE
    };

    let present_mode = get_swapchain_present_mode(&support.present_modes);

    let create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(image_count)
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        .pre_transform(support.capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true);

    let handle = devices.logical.create_swapchain_khr(&create_info, None)?;

    Ok(handle)
}

unsafe fn create_image_views(
    device: &Device,
    format: vk::Format,
    images: &Vec<vk::Image>,
) -> anyhow::Result<Vec<vk::ImageView>> {
    let image_views = images
        .iter()
        .map(|i| {
            let components = vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY);

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let info = vk::ImageViewCreateInfo::builder()
                .image(*i)
                .view_type(vk::ImageViewType::_2D)
                .format(format)
                .components(components)
                .subresource_range(subresource_range);

            device.create_image_view(&info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(image_views)
}

fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

fn get_swapchain_present_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    modes
        .iter()
        .cloned()
        .find(|m| *m == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO)
}

fn get_swapchain_extent(
    window_size: (u32, u32),
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    let (width, height) = window_size;

    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let clamp = |value: u32, min: u32, max: u32| min.max(max.min(value));
        vk::Extent2D::builder()
            .width(clamp(
                width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ))
            .height(clamp(
                height,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ))
            .build()
    }
}
