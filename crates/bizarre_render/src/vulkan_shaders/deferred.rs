use nalgebra_glm::{Mat4, Vec4};

use crate::{
    material::binding::{BindingType, MaterialBinding, MaterialBindingRate},
    vulkan_utils::shader::ShaderStage,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ubo {
    pub view_projection: Mat4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Transform {
    pub transform: Mat4,
}

impl From<Mat4> for Transform {
    fn from(value: Mat4) -> Self {
        Self {
            transform: value,
            ..Default::default()
        }
    }
}

pub const fn material_bindings() -> [MaterialBinding; 2] {
    [
        MaterialBinding {
            binding: 0,
            binding_type: BindingType::UniformBuffer,
            shader_stage: ShaderStage::Vertex,
            binding_rate: MaterialBindingRate::PerFrame,
            set: 0,
        },
        MaterialBinding {
            binding: 1,
            set: 0,
            binding_type: BindingType::StorageBuffer,
            shader_stage: ShaderStage::Vertex,
            binding_rate: MaterialBindingRate::PerFrame,
        },
    ]
}
