use std::path::PathBuf;

use ash::vk;

use crate::vulkan_utils::shader::ShaderStage;

use super::{binding::MaterialBinding, pipeline_features::PipelineFeatures};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaterialPassType {
    Geometry,
    Lighting,
    Translucent,
}

pub struct MaterialPassStage {
    pub shader_path: PathBuf,
    pub shader_stage: ShaderStage,
}

pub struct MaterialPass {
    pub pipeline: vk::Pipeline,
    pub set_layout: vk::DescriptorSetLayout,
    pub bindings: Box<[MaterialBinding]>,
}

pub struct MaterialPassCreateInfo<'a> {
    pub stages: MaterialPassStage,
    pub pipeline_features: PipelineFeatures,
    pub bindings: &'a Vec<MaterialBinding>,
    pub pass_type: MaterialPassType,
}
