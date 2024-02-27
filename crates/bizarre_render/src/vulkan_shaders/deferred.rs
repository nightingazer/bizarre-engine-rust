use ash::vk;
use nalgebra_glm::Mat4;

use crate::{
    material::binding::{BindingType, MaterialBinding},
    vulkan_utils::shader::ShaderStage,
};

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

pub fn material_bindings() -> [MaterialBinding; 1] {
    [MaterialBinding {
        binding: 0,
        binding_type: BindingType::UniformBuffer,
        shader_stage: ShaderStage::Vertex,
        set: 0,
    }]
}
