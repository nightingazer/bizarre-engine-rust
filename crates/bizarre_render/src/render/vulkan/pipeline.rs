use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline,
    },
    render_pass::{RenderPass, Subpass},
    shader::ShaderModule,
};

use super::vertex::VulkanVertexData;

pub fn create_graphics_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: &Arc<RenderPass>,
    device: &Arc<Device>,
) -> Result<Arc<GraphicsPipeline>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(VulkanVertexData::per_vertex())
        .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())?;
    Ok(pipeline)
}
