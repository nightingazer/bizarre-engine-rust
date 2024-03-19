use nalgebra_glm::{Mat4, Vec3};

use crate::{
    material::binding::{BindingType, MaterialBinding, MaterialBindingRate},
    render_math::DirectionalLight,
    vulkan_utils::shader::ShaderStage,
};

use super::shader_common;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct AmbientUbo {
    pub ambient_color: Vec3,
}

pub const fn ambient_material_bindings() -> [MaterialBinding; 3] {
    [
        MaterialBinding {
            binding: 0,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::UniformBuffer,
            binding_rate: MaterialBindingRate::PerFrame,
        },
        MaterialBinding {
            binding: 1,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
            binding_rate: MaterialBindingRate::PerFrame,
        },
        MaterialBinding {
            binding: 2,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
            binding_rate: MaterialBindingRate::PerFrame,
        },
    ]
}

pub type DierctionalUbo = shader_common::ViewProjection;

#[repr(C)]
#[derive(Debug, Clone, Default, Copy)]
pub struct DirectionalLightsSSBO {
    pub direction: Vec3,
    _pad0: f32,
    pub color: Vec3,
    _pad1: f32,
}

impl From<&DirectionalLight> for DirectionalLightsSSBO {
    fn from(value: &DirectionalLight) -> Self {
        Self {
            color: value.color,
            direction: value.direction,
            ..Default::default()
        }
    }
}

pub const fn directional_material_bindings() -> [MaterialBinding; 4] {
    [
        MaterialBinding {
            binding: 0,
            set: 0,
            shader_stage: ShaderStage::Vertex,
            binding_type: BindingType::UniformBuffer,
            binding_rate: MaterialBindingRate::PerFrame,
        },
        MaterialBinding {
            binding: 1,
            set: 0,
            shader_stage: ShaderStage::Vertex,
            binding_type: BindingType::StorageBuffer,
            binding_rate: MaterialBindingRate::PerFrame,
        },
        MaterialBinding {
            binding: 2,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
            binding_rate: MaterialBindingRate::PerFrame,
        },
        MaterialBinding {
            binding: 3,
            set: 0,
            shader_stage: ShaderStage::Fragment,
            binding_type: BindingType::InputAttachment,
            binding_rate: MaterialBindingRate::PerFrame,
        },
    ]
}
