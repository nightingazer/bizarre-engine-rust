use std::path::PathBuf;

use ash::vk::ShaderRequiredSubgroupSizeCreateInfoEXT;

use crate::{
    vulkan::{
        device::VulkanDevice,
        pipeline::{VulkanPipeline, VulkanPipelineRequirements},
    },
    vulkan_utils::shader::ShaderStage,
};

use super::binding::{MaterialBinding, MaterialBindingRate};

#[derive(Debug, Clone)]
pub struct MaterialPassStage {
    pub shader_path: PathBuf,
    pub shader_stage: ShaderStage,
}
