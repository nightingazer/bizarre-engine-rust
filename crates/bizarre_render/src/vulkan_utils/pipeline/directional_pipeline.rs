use std::{ffi::CStr, path::Path};

use anyhow::Result;
use ash::vk;

use crate::{
    vertex::PositionVertex,
    vulkan::pipeline::VulkanPipeline,
    vulkan_shaders::directional,
    vulkan_utils::shader::{load_shader, ShaderType},
};

pub fn create_directional_pipeline(
    viewport: &vk::Viewport,
    render_pass: vk::RenderPass,
    device: &ash::Device,
) -> Result<VulkanPipeline> {
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_state_info =
        vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

    let vertex_binding_descriptions = [vk::VertexInputBindingDescription::builder()
        .binding(0)
        .input_rate(vk::VertexInputRate::VERTEX)
        .stride(std::mem::size_of::<PositionVertex>() as u32)
        .build()];

    let vertex_input_attributes = PositionVertex::attribute_description();

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&vertex_binding_descriptions)
        .vertex_attribute_descriptions(&vertex_input_attributes);

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_FAN)
        .primitive_restart_enable(false);

    let scissors = [vk::Rect2D {
        extent: vk::Extent2D {
            width: viewport.width as u32,
            height: viewport.height as u32,
        },
        offset: vk::Offset2D { x: 0, y: 0 },
    }];

    let viewports = [*viewport];
    let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&viewports)
        .scissors(&scissors);

    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .color_blend_op(vk::BlendOp::ADD)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::MAX)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .build()];

    let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&color_blend_attachments);

    let set_layout = {
        let set_bindings = directional::descriptor_set_bindings();
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&set_bindings);
        unsafe { device.create_descriptor_set_layout(&create_info, None)? }
    };

    let layout = {
        let layouts = [set_layout];
        let layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&layouts);
        unsafe { device.create_pipeline_layout(&layout_info, None)? }
    };

    let (vert_module, vert_stage) = {
        let code = load_shader(
            Path::new("assets/shaders/directional.vert"),
            ShaderType::Vertex,
        )?;

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        let module = unsafe { device.create_shader_module(&create_info, None)? };

        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
            .build();

        (module, stage)
    };

    let (frag_module, frag_stage) = {
        let code = load_shader(
            Path::new("assets/shaders/directional.frag"),
            ShaderType::Fragment,
        )?;

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);

        let module = unsafe { device.create_shader_module(&create_info, None)? };

        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
            .build();

        (module, stage)
    };

    let stages = [vert_stage, frag_stage];

    let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

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
        .render_pass(render_pass)
        .subpass(1)
        .build();

    let pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
            .map_err(|(_, e)| e)?
    };

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    Ok(VulkanPipeline {
        handle: pipeline[0],
        layout,
        set_layout,
    })
}
