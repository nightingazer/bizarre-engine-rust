use anyhow::Result;
use ash::vk;

use crate::{
    vertex::{MeshVertex, Vertex},
    vulkan::{
        device::VulkanDevice,
        pipeline::{VulkanPipelineRequirements, VulkanPipelineStage},
    },
    vulkan_utils::shader::ShaderStage,
};

use super::{
    pipeline_features::{CullMode, PipelineFeatureFlags, PipelineFeatures, PrimitiveTopology},
    Material, MaterialType,
};

pub fn default_plain(
    sample_count: vk::SampleCountFlags,
    render_pass: vk::RenderPass,
    device: &VulkanDevice,
) -> Result<Material> {
    let deferred_reqs = VulkanPipelineRequirements {
        attachment_count: 2,
        bindings: &[],
        features: PipelineFeatures {
            culling: CullMode::Back,
            flags: PipelineFeatureFlags::DEPTH_TEST | PipelineFeatureFlags::DEPTH_WRITE,
            ..Default::default()
        },
        render_pass,
        material_type: MaterialType::Opaque,
        sample_count,
        stage_definitions: &[
            VulkanPipelineStage {
                path: String::from("assets/shaders/deferred.vert"),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: String::from("assets/shaders/deferred.frag"),
                stage: ShaderStage::Fragment,
            },
        ],
        base_pipeline: None,
        vertex_attributes: MeshVertex::attribute_description(),
        vertex_bindings: MeshVertex::binding_description(),
    };

    Material::new(&deferred_reqs, device)
}
