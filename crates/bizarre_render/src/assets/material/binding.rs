use ash::vk;

use crate::vulkan_utils::shader::ShaderStage;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
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
#[derive(Debug, Clone)]
pub struct MaterialBinding {
    pub binding: u32,
    pub set: u32,
    pub shader_stage: ShaderStage,
    pub binding_type: BindingType,
}

impl From<&MaterialBinding> for vk::DescriptorSetLayoutBinding {
    fn from(value: &MaterialBinding) -> Self {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(value.binding)
            .descriptor_count(1)
            .stage_flags(value.shader_stage.into())
            .descriptor_type(value.binding_type.into())
            .build()
    }
}
