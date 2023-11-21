use std::sync::Arc;

use anyhow::{bail, Result};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        self, AutoCommandBufferBuilder, CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    format::Format,
    image::{
        sys::ImageCreateInfo,
        view::{ImageView, ImageViewCreateInfo, ImageViewType},
        Image, ImageCreateFlags, ImageLayout, ImageSubresourceRange, ImageType, ImageUsage,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

use crate::cube_map::{self, CubeMap};

pub struct VulkanCubeMap {
    pub texture: Arc<ImageView>,
}

impl VulkanCubeMap {
    pub fn new(
        cube_map: CubeMap,
        memory_allocator: Arc<StandardMemoryAllocator>,
        queue_family_indices: &[u32],
        cmd_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<Self> {
        let texture_size = cube_map
            .texture_data
            .iter()
            .fold(0_usize, |acc, e| acc + e.len());

        if texture_size == 0 {
            bail!("Cube map texture data is empty");
        }

        let data = cube_map.texture_data.iter().fold(
            Vec::<u8>::with_capacity(texture_size),
            |mut acc, e| {
                acc.extend(e);
                acc
            },
        );

        let format = Format::R8G8B8A8_SRGB;

        let image = Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                array_layers: 6,
                extent: [cube_map.side_width, cube_map.side_height, 1],
                image_type: ImageType::Dim2d,
                flags: ImageCreateFlags::CUBE_COMPATIBLE,
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
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            data.clone(),
        )?;

        cmd_buffer
            .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image.clone()))?;

        let view = ImageView::new(
            image.clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Cube,
                subresource_range: ImageSubresourceRange::from_parameters(format, 1, 6),
                format,
                usage: ImageUsage::SAMPLED,
                ..Default::default()
            },
        )?;

        Ok(Self { texture: view })
    }
}
