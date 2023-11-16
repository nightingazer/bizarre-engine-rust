use std::sync::Arc;

use anyhow::{bail, Result};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        self, AutoCommandBufferBuilder, CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    format::Format,
    image::{
        sys::{Image, ImageCreateInfo},
        view::{ImageView, ImageViewCreateInfo},
        ImageCreateFlags, ImageDimensions, ImageLayout, ImageSubresourceRange, ImageUsage,
        ImageViewType, ImmutableImage, MipmapsCount,
    },
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
};

use crate::cube_map::{self, CubeMap};

pub struct VulkanCubeMap {
    pub texture: Arc<ImageView<ImmutableImage>>,
}

impl VulkanCubeMap {
    pub fn new(
        cube_map: CubeMap,
        memory_allocator: &StandardMemoryAllocator,
        queue_family_indices: &[u32],
        cmd_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<Self> {
        let dimensions = ImageDimensions::Dim2d {
            width: cube_map.side_width,
            height: cube_map.side_height,
            array_layers: 6,
        };

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

        let (image, image_init) = ImmutableImage::uninitialized(
            memory_allocator,
            dimensions,
            format,
            MipmapsCount::One,
            ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            ImageCreateFlags::CUBE_COMPATIBLE,
            ImageLayout::ShaderReadOnlyOptimal,
            queue_family_indices.iter().cloned(),
        )?;

        let buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            data.clone(),
        )?;

        cmd_buffer.copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(buffer, image_init));

        // let image = ImmutableImage::from_iter(
        //     memory_allocator,
        //     data,
        //     dimensions,
        //     MipmapsCount::One,
        //     Format::R8G8B8A8_SRGB,
        //     cmd_buffer,
        // )?;

        let view = ImageView::new(
            image.clone(),
            ImageViewCreateInfo {
                view_type: ImageViewType::Cube,
                subresource_range: ImageSubresourceRange::from_parameters(format, 1, 6),
                format: Some(format),
                usage: ImageUsage::SAMPLED,
                ..Default::default()
            },
        )?;

        Ok(Self { texture: view })
    }
}
