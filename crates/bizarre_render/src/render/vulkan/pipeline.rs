use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
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
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(VulkanVertexData::per_vertex())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
        .render_pass(render_pass)
        .build(device.clone())?;
    Ok(pipeline)
}
