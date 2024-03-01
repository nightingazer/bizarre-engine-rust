use std::ops::{Deref, DerefMut};

use anyhow::Result;
use ash::vk;

pub struct VulkanRenderPass {
    pub handle: vk::RenderPass,
    pub output_attachment: vk::AttachmentDescription,
    pub depth_attachment: vk::AttachmentDescription,
    pub color_attachment: vk::AttachmentDescription,
    pub normals_attachment: vk::AttachmentDescription,
}

impl VulkanRenderPass {
    pub fn new(output_format: vk::Format, device: &ash::Device) -> Result<Self> {
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

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(vk::Format::D32_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachment = vk::AttachmentDescription::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let normals_attachment = vk::AttachmentDescription::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let output_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let deferred_color_attachment_ref = vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let deferred_normals_attachment_ref = vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let lighting_color_attachment_ref = vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        let lighting_normals_attachment_ref = vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        let deferred_attachments = [
            deferred_color_attachment_ref,
            deferred_normals_attachment_ref,
        ];
        let deferred_subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&deferred_attachments)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();

        let lighting_attachments = [output_attachment_ref];
        let lighting_input_attachments = [
            lighting_color_attachment_ref,
            lighting_normals_attachment_ref,
        ];
        let lighting_subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&lighting_attachments)
            .input_attachments(&lighting_input_attachments)
            .build();

        let floor_subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&lighting_attachments)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ..Default::default()
            },
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: 1,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::BY_REGION,
                ..Default::default()
            },
            vk::SubpassDependency {
                src_subpass: 1,
                dst_subpass: 2,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
                ..Default::default()
            },
        ];

        let subpasses = [deferred_subpass, lighting_subpass, floor_subpass];
        let attachments = [
            output_attachment,
            depth_attachment,
            color_attachment,
            normals_attachment,
        ];

        let render_pass = {
            let create_info = vk::RenderPassCreateInfo::builder()
                .attachments(&attachments)
                .subpasses(&subpasses)
                .dependencies(&dependencies)
                .build();

            unsafe { device.create_render_pass(&create_info, None)? }
        };

        Ok(Self {
            handle: render_pass,
            output_attachment,
            depth_attachment,
            color_attachment,
            normals_attachment,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_render_pass(self.handle, None);
        }
    }
}

impl Deref for VulkanRenderPass {
    type Target = vk::RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl DerefMut for VulkanRenderPass {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}
