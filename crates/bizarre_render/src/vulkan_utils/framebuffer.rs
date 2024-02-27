use anyhow::Result;
use ash::vk;

use crate::global_context::VULKAN_GLOBAL_CONTEXT;

pub fn create_framebuffer(
    attachments: &[vk::ImageView],
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
) -> Result<vk::Framebuffer> {
    let device = VULKAN_GLOBAL_CONTEXT.device();
    let create_info = vk::FramebufferCreateInfo::builder()
        .attachments(attachments)
        .height(extent.height)
        .width(extent.width)
        .layers(1)
        .render_pass(render_pass)
        .build();

    let framebuffer = unsafe { device.create_framebuffer(&create_info, None)? };
    Ok(framebuffer)
}
