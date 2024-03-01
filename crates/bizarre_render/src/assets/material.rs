use std::sync::Arc;

use anyhow::{Error, Result};
use ash::vk;

use crate::{
    global_context::VULKAN_GLOBAL_CONTEXT,
    vulkan::pipeline::{VulkanPipeline, VulkanPipelineRequirements},
};

use self::{
    binding::MaterialBinding,
    pass::{MaterialPass, MaterialPassType},
};

pub mod binding;
pub mod builtin_materials;
pub mod pass;
pub mod pipeline_features;

pub const MATERIAL_PASS_COUNT: usize = std::mem::variant_count::<MaterialPassType>();

pub struct Material {
    pub passes: [Option<Box<[MaterialPass]>>; MATERIAL_PASS_COUNT],
}

impl Material {
    pub fn new(
        pass_requirements: [Option<&[VulkanPipelineRequirements]>; MATERIAL_PASS_COUNT],
    ) -> Result<Self> {
        let mut passes = pass_requirements
            .into_iter()
            .map(|reqs| match reqs {
                None => Ok::<Option<Box<[MaterialPass]>>, Error>(None),
                Some(reqs) => {
                    let passes = reqs
                        .iter()
                        .map(|req| {
                            let pipeline = VulkanPipeline::from_requirements(req)?;
                            let bindings = req.bindings.to_vec().into_boxed_slice();
                            let pass = MaterialPass { pipeline, bindings };
                            Ok(pass)
                        })
                        .collect::<Result<Vec<MaterialPass>>>()?
                        .into_boxed_slice();
                    Ok(Some(passes))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let mut result = {
            const REPEAT_VALUE: Option<Box<[MaterialPass]>> = None;
            [REPEAT_VALUE; MATERIAL_PASS_COUNT]
        };

        for i in 0..MATERIAL_PASS_COUNT {
            result[i] = passes[i].take();
        }

        Ok(Self { passes: result })
    }
}

pub struct MaterialInstance {
    pub material: Arc<Material>,
    pub descriptor_sets: [Option<Box<[vk::DescriptorSet]>>; MATERIAL_PASS_COUNT],
}
