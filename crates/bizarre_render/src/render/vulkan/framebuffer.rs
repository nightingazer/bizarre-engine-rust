use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, SwapchainImage},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

pub fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    allocator: &StandardMemoryAllocator,
) -> Result<(
    Vec<Arc<Framebuffer>>,
    Arc<ImageView<AttachmentImage>>,
    Arc<ImageView<AttachmentImage>>,
)> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, -(dimensions[1] as f32)];
    viewport.origin = [0.0, dimensions[1] as f32];

    let color_buffer = ImageView::new_default(AttachmentImage::transient_input_attachment(
        allocator,
        dimensions,
        Format::A2B10G10R10_UNORM_PACK32,
    )?)?;

    let normals_buffer = ImageView::new_default(AttachmentImage::transient_input_attachment(
        allocator,
        dimensions,
        Format::R16G16B16A16_SFLOAT,
    )?)?;

    let depth_buffer = ImageView::new_default(AttachmentImage::transient(
        allocator,
        dimensions,
        Format::D16_UNORM,
    )?)?;

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
