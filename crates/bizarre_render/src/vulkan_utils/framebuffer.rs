use anyhow::Result;
use ash::vk;

pub fn create_framebuffer(
    attachments: &[vk::ImageView],
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    device: &ash::Device,
) -> Result<vk::Framebuffer> {
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
