use ash::vk;
use nalgebra_glm::Vec3;

use crate::{
    material::binding::{BindingType, MaterialBinding},
    vulkan_utils::shader::ShaderStage,
};

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Ubo {
    pub direction: Vec3,
    pub _pad0: f32,
    pub color: Vec3,
}

pub const fn material_bindings() -> [MaterialBinding; 3] {
    [
        MaterialBinding {
            binding: 0,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::UniformBuffer,
        },
        MaterialBinding {
            binding: 1,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
        },
        MaterialBinding {
            binding: 2,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
        },
    ]
}
