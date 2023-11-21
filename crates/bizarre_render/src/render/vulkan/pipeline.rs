use std::sync::Arc;

use anyhow::{anyhow, Result};
use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::{CullMode, RasterizationState},
            subpass::PipelineSubpassType,
            vertex_input::{Vertex, VertexDefinition, VertexInputState},
            viewport::ViewportState,
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    shader::{EntryPoint, ShaderModule},
};

use super::vertex::{VulkanPosition2DVertex, VulkanVertex2D};

fn make_stages_and_layout(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
) -> Result<(
    [PipelineShaderStageCreateInfo; 2],
    [EntryPoint; 2],
    Arc<PipelineLayout>,
)> {
    let vs = vs.entry_point("main").ok_or(anyhow!(
        "Failed to create vertex shader entry point for vertex shader"
    ))?;

    let fs = fs.entry_point("main").ok_or(anyhow!(
        "Failed to create fragment shader entry point for fragment shader"
    ))?;

    let stages = [
        PipelineShaderStageCreateInfo::new(vs.clone()),
        PipelineShaderStageCreateInfo::new(fs.clone()),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())?,
    )?;

    Ok((stages, [vs, fs], layout))
}

pub fn create_editor_grid_graphics_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineSubpassType>,
    device: Arc<Device>,
    num_attachments: u32,
) -> Result<Arc<GraphicsPipeline>> {
    let (stages, _, layout) =
        make_stages_and_layout(device.clone(), vertex_shader, fragment_shader)?;

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            input_assembly_state: Some(InputAssemblyState::default()),
            vertex_input_state: Some(VertexInputState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None,
                ..Default::default()
            }),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                num_attachments,
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(render_pass.into().clone()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )?;

    Ok(pipeline)
}

pub fn create_skybox_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineSubpassType>,
    device: Arc<Device>,
) -> Result<Arc<GraphicsPipeline>> {
    let (stages, entry_points, layout) =
        make_stages_and_layout(device.clone(), vertex_shader, fragment_shader)?;

    let vs_entry = &entry_points[0];
    let vertex_input_state =
        Some(VulkanPosition2DVertex::per_vertex().definition(&vs_entry.info().input_interface)?);

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state,
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None,
                ..Default::default()
            }),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::LessOrEqual,
                }),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState::default(),
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(render_pass.into().clone()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )?;

    Ok(pipeline)
}

pub fn create_lighting_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineSubpassType>,
    device: Arc<Device>,
    num_attachments: u32,
) -> Result<Arc<GraphicsPipeline>> {
    let (stages, entry_points, layout) =
        make_stages_and_layout(device.clone(), vertex_shader, fragment_shader)?;

    let vs_entry = &entry_points[0];
    let vertex_input_state =
        Some(VulkanPosition2DVertex::per_vertex().definition(&vs_entry.info().input_interface)?);

    let create_info = GraphicsPipelineCreateInfo {
        stages: stages.into_iter().collect(),
        vertex_input_state,
        input_assembly_state: Some(InputAssemblyState::default()),
        viewport_state: Some(ViewportState::default()),
        rasterization_state: Some(RasterizationState {
            cull_mode: CullMode::Back,
            ..Default::default()
        }),
        multisample_state: Some(MultisampleState::default()),
        color_blend_state: Some(ColorBlendState::with_attachment_states(
            num_attachments,
            ColorBlendAttachmentState {
                blend: Some(AttachmentBlend {
                    src_color_blend_factor: BlendFactor::One,
                    dst_color_blend_factor: BlendFactor::One,
                    color_blend_op: BlendOp::Add,
                    src_alpha_blend_factor: BlendFactor::One,
                    dst_alpha_blend_factor: BlendFactor::One,
                    alpha_blend_op: BlendOp::Max,
                }),
                ..Default::default()
            },
        )),
        dynamic_state: [DynamicState::Viewport].into_iter().collect(),
        subpass: Some(render_pass.into().clone()),
        ..GraphicsPipelineCreateInfo::layout(layout)
    };

    let pipeline = GraphicsPipeline::new(device.clone(), None, create_info)?;
    Ok(pipeline)
}

pub fn create_deferred_pipeline<V: Vertex>(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineSubpassType>,
    device: Arc<Device>,
    num_attachments: u32,
) -> Result<Arc<GraphicsPipeline>> {
    let (stages, entry_points, layout) =
        make_stages_and_layout(device.clone(), vertex_shader, fragment_shader)?;

    let vs_entry = &entry_points[0];
    let vertex_input_state = Some(V::per_vertex().definition(&vs_entry.info().input_interface)?);

    let mut create_info = GraphicsPipelineCreateInfo {
        stages: stages.into_iter().collect(),
        vertex_input_state,
        input_assembly_state: Some(InputAssemblyState::default()),
        viewport_state: Some(ViewportState::default()),
        rasterization_state: Some(RasterizationState {
            cull_mode: CullMode::Back,
            ..Default::default()
        }),
        depth_stencil_state: Some(DepthStencilState {
            depth: Some(DepthState::simple()),
            ..Default::default()
        }),
        dynamic_state: [DynamicState::Viewport].into_iter().collect(),
        subpass: Some(render_pass.into().clone()),
        multisample_state: Some(MultisampleState::default()),
        color_blend_state: Some(ColorBlendState::with_attachment_states(
            num_attachments,
            ColorBlendAttachmentState::default(),
        )),
        ..GraphicsPipelineCreateInfo::layout(layout)
    };

    let pipeline = GraphicsPipeline::new(device.clone(), None, create_info)?;
    Ok(pipeline)
}

pub fn create_screen_text_pipeline(
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    render_pass: impl Into<PipelineSubpassType>,
    device: Arc<Device>,
) -> Result<Arc<GraphicsPipeline>> {
    let (stages, entry_points, layout) =
        make_stages_and_layout(device.clone(), vertex_shader, fragment_shader)?;

    let vs_entry = &entry_points[0];
    let vertex_input_state =
        Some(VulkanVertex2D::per_vertex().definition(&vs_entry.info().input_interface)?);

    let pipeline = GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state,
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::None,
                ..Default::default()
            }),
            depth_stencil_state: None,
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                1,
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(render_pass.into().clone()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )?;

    Ok(pipeline)
}
