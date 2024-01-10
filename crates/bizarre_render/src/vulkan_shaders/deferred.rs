use ash::vk;
use nalgebra_glm::Mat4;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ubo {
    pub view_projection: Mat4,
    pub model: [Mat4; 100],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VertexPushConstant {
    model_offset: u32,
}

pub fn descriptor_set_bindings() -> [vk::DescriptorSetLayoutBinding; 1] {
    [vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .descriptor_count(1)
        .build()]
}
