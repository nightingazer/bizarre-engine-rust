use std::ops::{Deref, DerefMut};

use anyhow::Result;
use ash::vk;

pub struct VulkanRenderPass {
    pub render_pass: vk::RenderPass,
    pub output_attachment: vk::AttachmentDescription,
}

impl VulkanRenderPass {
    pub fn new(
        output_format: vk::Format,
        extent: &vk::Extent2D,
        device: &ash::Device,
    ) -> Result<Self> {
        let output_attachment = vk::AttachmentDescription::builder()
            .format(output_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let output_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[output_attachment_ref])
            .build();

        let render_pass = {
            let create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&[output_attachment])
                .subpasses(&[subpass])
                .build();

            unsafe { device.create_render_pass(&create_info, None)? }
        };

        Ok(Self {
            render_pass,
            output_attachment,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}

impl Deref for VulkanRenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl DerefMut for VulkanRenderPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.render_pass
    }
}
