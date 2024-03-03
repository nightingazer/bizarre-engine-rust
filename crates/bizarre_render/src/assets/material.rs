use std::{collections::VecDeque, sync::Arc};

use anyhow::{anyhow, Error, Result};
use ash::vk;

use crate::{
    global_context::VULKAN_GLOBAL_CONTEXT,
    vulkan::pipeline::{VulkanPipeline, VulkanPipelineRequirements},
};

use self::{
    binding::MaterialBinding,
    pass::{MaterialPassType, MaterialPipeline},
};

pub mod binding;
pub mod builtin_materials;
pub mod pass;
pub mod pipeline_features;

pub const MATERIAL_PASS_COUNT: usize = std::mem::variant_count::<MaterialPassType>();

pub struct Material {
    pub passes: [Option<Box<[MaterialPipeline]>>; MATERIAL_PASS_COUNT],
}

impl Material {
    pub fn new(
        pass_requirements: [Option<&[VulkanPipelineRequirements]>; MATERIAL_PASS_COUNT],
    ) -> Result<Self> {
        let mut passes = pass_requirements
            .into_iter()
            .map(|reqs| match reqs {
                None => Ok::<Option<Box<[MaterialPipeline]>>, Error>(None),
                Some(reqs) => {
                    let passes = reqs
                        .iter()
                        .map(|req| {
                            let pipeline = VulkanPipeline::from_requirements(req)?;
                            let bindings = req.bindings.to_vec().into_boxed_slice();
                            let pass = MaterialPipeline { pipeline, bindings };
                            Ok(pass)
                        })
                        .collect::<Result<Vec<MaterialPipeline>>>()?
                        .into_boxed_slice();
                    Ok(Some(passes))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        let mut result = {
            const REPEAT_VALUE: Option<Box<[MaterialPipeline]>> = None;
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

impl MaterialInstance {
    pub fn new(material: Arc<Material>) -> Result<Self> {
        let layouts = material
            .passes
            .iter()
            .filter_map(|pipelines| match pipelines {
                None => None,
                Some(pipelines) => {
                    let layouts = pipelines
                        .iter()
                        .map(|pipeline| pipeline.pipeline.set_layout)
                        .collect::<Vec<_>>();
                    Some(layouts)
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(VULKAN_GLOBAL_CONTEXT.descriptor_pool())
            .set_layouts(layouts.as_slice());

        let sets = unsafe {
            VULKAN_GLOBAL_CONTEXT
                .device()
                .allocate_descriptor_sets(&allocate_info)?
        };

        let mut sets = VecDeque::from(sets);

        let mut sets = material.passes.iter().map(|pipelines| {
            match pipelines {
                None => Ok(None),
                Some(pipelines) => {
                    let sets = pipelines.iter().map(|_| {
                        sets.pop_front().ok_or(anyhow!("Failed to create material instance! There are less descriptor sets than necessary..."))
                    }).collect::<Result<Vec<_>>>()?.into_boxed_slice();
                    Ok(Some(sets))
                }
            }
        }).collect::<Result<Vec<_>>>()?;

        let mut set_array = {
            const REPEAT_VALUE: Option<Box<[vk::DescriptorSet]>> = None;
            [REPEAT_VALUE; MATERIAL_PASS_COUNT]
        };

        for i in 0..MATERIAL_PASS_COUNT {
            set_array[i] = sets[i].take();
        }

        Ok(Self {
            material,
            descriptor_sets: set_array,
        })
    }
}
