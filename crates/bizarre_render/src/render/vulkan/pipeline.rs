use vulkanalia::{bytecode::Bytecode, prelude::v1_2::*};

use super::{swapchain::VulkanSwapchain, vulkan_renderer::VulkanRenderContext};

#[derive(Debug)]
pub struct Pipeline {
    pub layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
}

impl Pipeline {
    pub unsafe fn new(swapchain: &VulkanSwapchain, device: &Device) -> anyhow::Result<Self> {
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder();
        let layout = device.create_pipeline_layout(&layout_create_info, None)?;

        let render_pass = create_render_pass(&swapchain.format.format, device)?;
        let pipeline = create_pipeline(layout, &swapchain.extent, render_pass, device)?;

        Ok(Self {
            layout,
            render_pass,
            pipeline,
        })
    }

    pub unsafe fn destroy(&self, device: &Device) {
        device.destroy_pipeline_layout(self.layout, None);
        device.destroy_render_pass(self.render_pass, None);
        device.destroy_pipeline(self.pipeline, None);
    }
}

unsafe fn create_pipeline(
    layout: vk::PipelineLayout,
    extent: &vk::Extent2D,
    render_pass: vk::RenderPass,
    device: &Device,
) -> anyhow::Result<vk::Pipeline> {
    let frag = include_bytes!("./../../../../../assets/shaders/base.frag.spv");
    let vert = include_bytes!("./../../../../../assets/shaders/base.vert.spv");

    let frag_module = create_shader_module(device, frag)?;
    let vert_module = create_shader_module(device, vert)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(b"main\0");

    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder();

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(extent.width as f32)
        .height(extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(*extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterizer_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false);

    let multisampling_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];

    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let _dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let stages = &[vert_stage, frag_stage];

    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .viewport_state(&viewport_state)
        .input_assembly_state(&input_assembly_state)
        .rasterization_state(&rasterizer_state)
        .multisample_state(&multisampling_state)
        .color_blend_state(&color_blend_state)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0)
        .base_pipeline_index(-1)
        .base_pipeline_handle(vk::Pipeline::null());

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    device.destroy_shader_module(frag_module, None);
    device.destroy_shader_module(vert_module, None);

    Ok(pipeline)
}

unsafe fn create_shader_module(
    device: &Device,
    bytecode: &[u8],
) -> anyhow::Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();
    let info = vk::ShaderModuleCreateInfo::builder()
        .code(bytecode.code())
        .code_size(bytecode.code_size());

    Ok(device.create_shader_module(&info, None)?)
}

unsafe fn create_render_pass(
    format: &vk::Format,
    device: &Device,
) -> anyhow::Result<vk::RenderPass> {
    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let color_attachment = vk::AttachmentDescription::builder()
        .format(*format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_attachments = &[color_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(color_attachments);

    let attachments = &[color_attachment];
    let subpasses = &[subpass];
    let dependencies = &[dependency];

    let info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses)
        .dependencies(dependencies);

    Ok(device.create_render_pass(&info, None)?)
}
