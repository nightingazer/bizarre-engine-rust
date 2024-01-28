use ash::vk;
use nalgebra_glm::Mat4;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Ubo {
    pub view: Mat4,
    pub projection: Mat4,
}

pub fn descriptor_set_bindings() -> [vk::DescriptorSetLayoutBinding; 1] {
    [vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_count(1)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .build()]
}
