use ash::vk;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ubo {
    pub color: [f32; 3],
    pub camera_forward: [f32; 3],
}

pub fn descriptor_set_bindings() -> [vk::DescriptorSetLayoutBinding; 3] {
    [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .descriptor_count(1)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .descriptor_count(1)
            .build(),
    ]
}
