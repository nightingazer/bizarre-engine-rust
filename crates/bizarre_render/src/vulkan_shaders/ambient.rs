use crate::{
    material::binding::{BindingType, MaterialBinding, MaterialBindingRate},
    vulkan_utils::shader::ShaderStage,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Ubo {
    pub color: [f32; 3],
}

pub const fn material_bindings() -> [MaterialBinding; 3] {
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
