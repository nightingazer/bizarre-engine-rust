use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{AutoCommandBufferBuilder, CopyBufferToImageInfo, PrimaryAutoCommandBuffer},
    format::Format,
    image::{
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageCreateInfo, ImageSubresourceRange, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::texture::Texture;

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

pub fn create_texture(
    texture: &Texture,
    allocator: Arc<StandardMemoryAllocator>,
    cmd_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
) -> Result<Arc<ImageView>> {
    let format = Format::R8G8B8A8_SRGB;

    let image = Image::new(
        allocator.clone(),
        ImageCreateInfo {
            extent: [texture.size.0, texture.size.1, 1],
            format,
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    )?;

    let buffer = Buffer::from_iter(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        texture.bytes.clone(),
    )?;

    cmd_buffer.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone()))?;

    let view = ImageView::new(
        image.clone(),
        ImageViewCreateInfo {
            view_type: ImageViewType::Dim2d,
            subresource_range: ImageSubresourceRange::from_parameters(format, 1, 1),
            format,
            ..Default::default()
        },
    )?;

    Ok(view)
}
