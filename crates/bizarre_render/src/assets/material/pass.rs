use std::path::PathBuf;

use crate::{vulkan::pipeline::VulkanPipeline, vulkan_utils::shader::ShaderStage};

use super::binding::MaterialBinding;

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

pub struct MaterialPipeline {
    pub pipeline: VulkanPipeline,
    pub bindings: Box<[MaterialBinding]>,
}
