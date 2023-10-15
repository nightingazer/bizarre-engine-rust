use super::vulkan_renderer::VulkanRenderContext;
use vulkanalia::prelude::v1_2::*;

pub unsafe fn create_framebuffers(
    image_views: &[vk::ImageView],
    extent: &vk::Extent2D,
    render_pass: vk::RenderPass,
    device: &Device,
) -> anyhow::Result<Vec<vk::Framebuffer>> {
    let framebuffers = image_views
        .iter()
        .map(|i| {
            let attachments = &[*i];
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(framebuffers)
}
