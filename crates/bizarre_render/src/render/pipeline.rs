use vulkanalia::{bytecode::Bytecode, prelude::v1_2::*};

pub struct Pipeline {
    pub layout: vk::PipelineLayout,
}

impl Pipeline {
    pub unsafe fn new(device: &Device, extent: &vk::Extent2D) -> anyhow::Result<Self> {
        let layout_create_info = vk::PipelineLayoutCreateInfo::builder();
        let layout = device.create_pipeline_layout(&layout_create_info, None)?;

        Ok(Self { layout })
    }

    pub unsafe fn destroy(&self, device: &Device) {
        device.destroy_pipeline_layout(self.layout, None);
    }
}

unsafe fn create_pipeline(device: &Device, extent: &vk::Extent2D) -> anyhow::Result<()> {
    let frag = include_bytes!("./../../../../assets/shaders/base.frag.spv");
    let vert = include_bytes!("./../../../../assets/shaders/base.vert.spv");

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
        .logic_op_enable(true)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let dynamic_states = &[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    device.destroy_shader_module(frag_module, None);
    device.destroy_shader_module(vert_module, None);

    Ok(())
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
