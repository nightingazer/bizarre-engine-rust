use anyhow::Result;
use ash::vk;

use crate::{
    material::pass::MaterialPassType,
    vertex::{MeshVertex, PositionVertex, Vertex},
    vulkan::pipeline::{VulkanPipelineRequirements, VulkanPipelineStage},
    vulkan_shaders::{ambient, deferred, directional, floor},
    vulkan_utils::shader::ShaderStage,
};

use super::{
    pipeline_features::{CullMode, PipelineFeatureFlags, PipelineFeatures, PrimitiveTopology},
    Material,
};

pub fn default_lighted(render_pass: vk::RenderPass) -> Result<Material> {
    let deferred_reqs = VulkanPipelineRequirements {
        attachment_count: 2,
        bindings: &deferred::material_bindings(),
        features: PipelineFeatures {
            culling: CullMode::Back,
            flags: PipelineFeatureFlags::DEPTH_TEST | PipelineFeatureFlags::DEPTH_WRITE,
            ..Default::default()
        },
        pass_type: MaterialPassType::Geometry,
        render_pass,
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

    let ambient_reqs = VulkanPipelineRequirements {
        attachment_count: 1,
        bindings: &ambient::material_bindings(),
        features: PipelineFeatures {
            flags: PipelineFeatureFlags::BLEND_ADD,
            primitive_topology: PrimitiveTopology::TriangleFan,
            ..deferred_reqs.features
        },
        pass_type: MaterialPassType::Lighting,
        stage_definitions: &[
            VulkanPipelineStage {
                path: String::from("assets/shaders/ambient.vert"),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: String::from("assets/shaders/ambient.frag"),
                stage: ShaderStage::Fragment,
            },
        ],
        vertex_attributes: PositionVertex::attribute_description(),
        vertex_bindings: PositionVertex::binding_description(),
        ..deferred_reqs.clone()
    };

    let directional_reqs = VulkanPipelineRequirements {
        bindings: &directional::material_bindings(),
        stage_definitions: &[
            VulkanPipelineStage {
                path: String::from("assets/shaders/directional.vert"),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: String::from("assets/shaders/directional.frag"),
                stage: ShaderStage::Fragment,
            },
        ],
        ..ambient_reqs.clone()
    };

    // let floor_req = VulkanPipelineRequirements {
    //     bindings: &floor::material_bindings(),
    //     pass_type: MaterialPassType::Translucent,
    //     features: PipelineFeatures {
    //         culling: CullMode::None,
    //         primitive_topology: PrimitiveTopology::TriangleFan,
    //         flags: PipelineFeatureFlags::BLEND_COLOR_ALPHA | PipelineFeatureFlags::DEPTH_TEST,
    //         ..Default::default()
    //     },
    //     stage_definitions: &[
    //         VulkanPipelineStage {
    //             path: String::from("assets/shaders/floor.vert"),
    //             stage: ShaderStage::Vertex,
    //         },
    //         VulkanPipelineStage {
    //             path: String::from("assets/shaders/floor.frag"),
    //             stage: ShaderStage::Fragment,
    //         },
    //     ],
    //     vertex_attributes: <() as Vertex>::attribute_description(),
    //     vertex_bindings: <() as Vertex>::binding_description(),
    //     ..directional_reqs.clone()
    // };

    Material::new([
        Some(&[deferred_reqs]),
        Some(&[ambient_reqs, directional_reqs]),
        None,
    ])
}
