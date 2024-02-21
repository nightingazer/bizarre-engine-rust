use ash::vk;

use crate::vulkan_utils::shader::ShaderStage;

#[repr(i32)]
pub enum BindingType {
    UniformBuffer = 6,
    InputAttachment = 10,
}

impl From<BindingType> for vk::DescriptorType {
    fn from(value: BindingType) -> Self {
        vk::DescriptorType::from_raw(value as i32)
    }
}

/// Describes a binding from the shader perspective
pub struct Binding {
    pub binding: i32,
    pub set: i32,
    pub shader_stage: ShaderStage,
}
