use std::sync::Arc;

use anyhow::{anyhow, Result};
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, BlendFactor, BlendOp, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            render_pass::PipelineRenderPassType,
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, StateMode,
    },
    shader::ShaderModule,
};

use super::vertex::{DummyVertexData, VulkanPositionVertexData, VulkanVertexData};

pub fn create_editor_grid_graphics_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineRenderPassType>,
    device: Arc<Device>,
    num_attachments: u32,
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::None))
        .color_blend_state(ColorBlendState::new(num_attachments).blend_alpha())
        .render_pass(render_pass)
        .build(device.clone())?;

    Ok(pipeline)
}

pub fn create_skybox_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineRenderPassType>,
    device: Arc<Device>,
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(DummyVertexData::per_vertex())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState {
            depth: Some(DepthState {
                write_enable: StateMode::Fixed(true),
                compare_op: StateMode::Fixed(CompareOp::LessOrEqual),
                ..Default::default()
            }),
            ..Default::default()
        })
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::None))
        .render_pass(render_pass)
        .build(device.clone())?;

    Ok(pipeline)
}

pub fn create_graphics_pipeline<V: Vertex>(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineRenderPassType>,
    device: Arc<Device>,
    color_op: Option<bool>,
    num_attachments: Option<u32>,
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(V::per_vertex())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
        .depth_stencil_state(DepthStencilState::simple_depth_test())
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
