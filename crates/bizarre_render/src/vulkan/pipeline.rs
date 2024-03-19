use core::slice::SlicePattern;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::CStr,
    path::Path,
};

use anyhow::{anyhow, Result};
use ash::vk;
use bizarre_logger::core_warn;

use crate::{
    material::{
        binding::{bindings_into_layouts, MaterialBinding},
        pipeline_features::{PipelineFeatureFlags, PipelineFeatures},
        MaterialType,
    },
    vulkan_utils::shader::{load_shader, ShaderStage},
};

use super::device::VulkanDevice;

#[derive(Debug)]
pub struct VulkanPipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub set_layouts: Box<[vk::DescriptorSetLayout]>,
}

#[derive(Debug, Clone)]
pub struct VulkanPipelineStage {
    pub path: String,
    pub stage: ShaderStage,
}

#[derive(Debug, Clone)]
pub struct VulkanPipelineRequirements<'a> {
    pub features: PipelineFeatures,
    pub material_type: MaterialType,
    pub bindings: &'a [MaterialBinding],
    pub stage_definitions: &'a [VulkanPipelineStage],
    pub attachment_count: usize,
    pub render_pass: vk::RenderPass,
    pub base_pipeline: Option<&'a VulkanPipeline>,
    pub vertex_bindings: Box<[vk::VertexInputBindingDescription]>,
    pub vertex_attributes: Box<[vk::VertexInputAttributeDescription]>,
    pub sample_count: vk::SampleCountFlags,
}

impl VulkanPipeline {
    pub fn from_requirements(
        requirements: &VulkanPipelineRequirements,
        device: &VulkanDevice,
    ) -> Result<VulkanPipeline> {
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let vertex_binding_descriptions = requirements.vertex_bindings.as_slice();
        let vertex_input_attributes = requirements.vertex_attributes.as_slice();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_input_attributes);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(requirements.features.primitive_topology.into())
            .primitive_restart_enable(false);

        let scissors = [vk::Rect2D::default()];
        let viewports = [vk::Viewport::default()];

        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(requirements.features.polygon_mode.into())
            .line_width(1.0)
            .cull_mode(requirements.features.culling.into())
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(requirements.sample_count);

        let color_blend_attachments = {
            let mut attachments = Vec::with_capacity(requirements.attachment_count);
            let mut blend_state = vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA);

            let feature_flags = requirements.features.flags;

            if feature_flags.intersects(PipelineFeatureFlags::BLEND_MASK) {
                blend_state = blend_state.blend_enable(true);

                if feature_flags.contains(PipelineFeatureFlags::BLEND_COLOR) {
                    blend_state = blend_state
                        .color_blend_op(vk::BlendOp::ADD)
                        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA);
                } else {
                    blend_state = blend_state
                        .color_blend_op(vk::BlendOp::MAX)
                        .src_color_blend_factor(vk::BlendFactor::ONE)
                        .dst_color_blend_factor(vk::BlendFactor::ZERO);
                }

                if feature_flags.contains(PipelineFeatureFlags::BLEND_ALPHA) {
                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::ADD)
                        .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA);
                } else {
                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::MAX)
                        .src_alpha_blend_factor(vk::BlendFactor::ONE)
                        .dst_alpha_blend_factor(vk::BlendFactor::ZERO);
                }

                if feature_flags.contains(PipelineFeatureFlags::BLEND_ADD) {
                    #[cfg(debug_assertions)]
                    if feature_flags.intersects(PipelineFeatureFlags::BLEND_COLOR_ALPHA) {
                        core_warn!(
                            "Pipeline is being created with BLEND_ADD and BLEND_COLOR/BLEND_ALPHA at the same time. Additional blending is being used"
                        );
                    }

                    blend_state = blend_state
                        .alpha_blend_op(vk::BlendOp::ADD)
                        .src_alpha_blend_factor(vk::BlendFactor::ONE)
                        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
                        .color_blend_op(vk::BlendOp::ADD)
                        .src_color_blend_factor(vk::BlendFactor::ONE)
                        .dst_color_blend_factor(vk::BlendFactor::ONE);
                }
            } else {
                blend_state = blend_state.blend_enable(false)
            }

            for _ in 0..requirements.attachment_count {
                attachments.push(blend_state.clone());
            }

            attachments
        };

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        let set_layouts = bindings_into_layouts(requirements.bindings, device)?;

        let layout = {
            let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts);
            unsafe { device.create_pipeline_layout(&layout_info, None)? }
        };

        let (modules, stages): (Vec<_>, Vec<_>) = requirements
            .stage_definitions
            .iter()
            .map(|VulkanPipelineStage { path, stage }| {
                let code = load_shader(Path::new(path), *stage)?;

                let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

                let module = unsafe { device.create_shader_module(&create_info, None)? };

                let stage = vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::from(*stage))
                    .module(module)
                    .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                    .build();

                Ok((module, stage))
            })
            .collect::<Result<Vec<(_, _)>>>()?
            .into_iter()
            .unzip();

        let mut depth_stencil_info =
            vk::PipelineDepthStencilStateCreateInfo::builder().depth_bounds_test_enable(false);

        if requirements.features.flags & PipelineFeatureFlags::DEPTH_MASK
            != PipelineFeatureFlags::empty()
        {
            depth_stencil_info = depth_stencil_info.depth_compare_op(vk::CompareOp::LESS);
            if requirements.features.flags & PipelineFeatureFlags::DEPTH_TEST
                != PipelineFeatureFlags::empty()
            {
                depth_stencil_info = depth_stencil_info.depth_test_enable(true);
            }
            if requirements.features.flags & PipelineFeatureFlags::DEPTH_WRITE
                != PipelineFeatureFlags::empty()
            {
                depth_stencil_info = depth_stencil_info.depth_write_enable(true);
            }
            if requirements.features.flags & PipelineFeatureFlags::STENCIL_TEST
                != PipelineFeatureFlags::empty()
            {
                depth_stencil_info = depth_stencil_info.stencil_test_enable(true);
            }
        }

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .depth_stencil_state(&depth_stencil_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state_info)
            .layout(layout)
            .render_pass(requirements.render_pass)
            .subpass(requirements.material_type as u32)
            .build();

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
                .map_err(|(_, e)| e)?
        };

        for module in modules {
            unsafe {
                device.destroy_shader_module(module, None);
            }
        }

        Ok(VulkanPipeline {
            handle: pipeline[0],
            layout,
            set_layouts: set_layouts.into_boxed_slice(),
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_pipeline_layout(self.layout, None);
            self.layout = vk::PipelineLayout::null();

            for layout in self.set_layouts.iter_mut() {
                device.destroy_descriptor_set_layout(*layout, None);
                *layout = vk::DescriptorSetLayout::null();
            }

            device.destroy_pipeline(self.handle, None);
            self.handle = vk::Pipeline::null();
        }
    }
}
