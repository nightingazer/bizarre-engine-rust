use std::sync::Arc;

use anyhow::{anyhow, Result};
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, BlendFactor, BlendOp, ColorBlendState},
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, RasterizationState},
            render_pass::PipelineRenderPassType,
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline,
    },
    shader::ShaderModule,
};

use super::vertex::VulkanVertexData;

pub fn create_graphics_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineRenderPassType>,
    device: Arc<Device>,
    color_op: Option<bool>,
    num_attachments: Option<u32>,
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(VulkanVertexData::per_vertex())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
        .render_pass(render_pass);

    match color_op {
        Some(color_op) if color_op => {
            let num_attachments = num_attachments.ok_or(anyhow!(
                "num_attachments must be specified if color_op is true"
            ))?;
            let result = pipeline
                .color_blend_state(
                    ColorBlendState::new(num_attachments).blend(AttachmentBlend {
                        color_op: BlendOp::Add,
                        color_source: BlendFactor::One,
                        color_destination: BlendFactor::One,
                        alpha_op: BlendOp::Max,
                        alpha_source: BlendFactor::One,
                        alpha_destination: BlendFactor::One,
                    }),
                )
                .build(device.clone())?;
            Ok(result)
        }
        _ => Ok(pipeline.build(device.clone())?),
    }
}
