use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    format::Format,
    image::{view::ImageView, Image},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

use super::vulkan_image::{create_color_attachment, create_depth_attachment};

pub fn window_size_dependent_setup(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    allocator: Arc<StandardMemoryAllocator>,
) -> Result<(Vec<Arc<Framebuffer>>, Arc<ImageView>, Arc<ImageView>)> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, -(extent[1] as f32)];
    viewport.offset = [0.0, extent[1] as f32];

    let color_buffer =
        create_color_attachment(allocator.clone(), extent, Format::A2B10G10R10_UNORM_PACK32)?;

    let normals_buffer =
        create_color_attachment(allocator.clone(), extent, Format::R16G16B16A16_SFLOAT)?;

    let depth_buffer = create_depth_attachment(allocator.clone(), extent, Format::D16_UNORM)?;

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;
            let r = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        view,
                        color_buffer.clone(),
                        normals_buffer.clone(),
                        depth_buffer.clone(),
                    ],
                    ..Default::default()
                },
            )?;
            Ok(r)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((framebuffers, color_buffer, normals_buffer))
}
