use std::path::PathBuf;

use anyhow::Result;
use ash::vk;

use crate::{
    vulkan::pipeline::{VulkanPipeline, VulkanPipelineRequirements},
    vulkan_utils::shader::ShaderStage,
};

use super::{binding::MaterialBinding, pipeline_features::PipelineFeatures};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum MaterialPassType {
    #[default]
    Geometry,
    Lighting,
    Translucent,
}

#[derive(Debug, Clone)]
pub struct MaterialPassStage {
    pub shader_path: PathBuf,
    pub shader_stage: ShaderStage,
}

pub struct MaterialPass {
    pub pipeline: VulkanPipeline,
    pub bindings: Box<[MaterialBinding]>,
}
