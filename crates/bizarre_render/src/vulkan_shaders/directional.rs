use ash::vk;
use nalgebra_glm::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Ubo {
    pub direction: Vec3,
    pub _pad0: f32,
    pub color: Vec3,
}

pub fn descriptor_set_bindings() -> [vk::DescriptorSetLayoutBinding; 3] {
    [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
    ]
}
