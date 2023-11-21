use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageCreateInfo, ImageSubresourceRange, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

pub fn create_color_attachment(
    allocator: Arc<StandardMemoryAllocator>,
    extent: [u32; 3],
    format: Format,
) -> Result<Arc<ImageView>> {
    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            extent,
            format,
            usage: ImageUsage::COLOR_ATTACHMENT
                | ImageUsage::TRANSIENT_ATTACHMENT
                | ImageUsage::INPUT_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )?;

    let image_view = ImageView::new(
        image,
        ImageViewCreateInfo {
            subresource_range: ImageSubresourceRange::from_parameters(format, 1, 1),
            usage: ImageUsage::COLOR_ATTACHMENT
                | ImageUsage::TRANSIENT_ATTACHMENT
                | ImageUsage::INPUT_ATTACHMENT,
            format,
            ..Default::default()
        },
    )?;

    Ok(image_view)
}

pub fn create_depth_attachment(
    allocator: Arc<StandardMemoryAllocator>,
    extent: [u32; 3],
    format: Format,
) -> Result<Arc<ImageView>> {
    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            extent,
            format,
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )?;

    let image_view = ImageView::new(
        image,
        ImageViewCreateInfo {
            subresource_range: ImageSubresourceRange::from_parameters(format, 1, 1),
            usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
            format,
            ..Default::default()
        },
    )?;

    Ok(image_view)
}
