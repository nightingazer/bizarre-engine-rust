use nalgebra_glm::Mat4;

use crate::{
    material::binding::{BindingType, MaterialBinding, MaterialBindingRate},
    vulkan_utils::shader::ShaderStage,
};

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Ubo {
    pub view: Mat4,
    pub projection: Mat4,
}

pub fn material_bindings() -> [MaterialBinding; 1] {
    [MaterialBinding {
        binding: 0,
        set: 0,
        binding_type: BindingType::UniformBuffer,
        shader_stage: ShaderStage::Vertex,
        binding_rate: MaterialBindingRate::PerFrame,
    }]
}
