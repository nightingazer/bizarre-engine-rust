use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use bizarre_logger::core_critical;
use specs::{hibitset::BitSetLike, BitSet};

use crate::vulkan::{
    device::VulkanDevice,
    pipeline::{VulkanPipeline, VulkanPipelineRequirements},
};

use self::binding::{binding_sets, BindObject, BindObjectSet, BindingSet, MaterialBindingRate};

pub mod binding;
pub mod builtin_materials;
pub mod pass;
pub mod pipeline_features;

#[derive(Debug, Clone, Copy)]
pub enum MaterialType {
    Opaque,
    Lighting,
    Translucent,
    Postprocess,
}

pub struct Material {
    pub material_type: MaterialType,
    pub pipeline: VulkanPipeline,
    pub per_instance_binding_sets: Box<[BindingSet]>,
    pub per_frame_binding_sets: Box<[BindingSet]>,
}

impl Material {
    pub fn new(requirements: &VulkanPipelineRequirements, device: &VulkanDevice) -> Result<Self> {
        let pipeline = VulkanPipeline::from_requirements(requirements, device)?;
        let (per_pass, per_frame) = requirements.bindings.iter().cloned().fold(
            (Vec::new(), Vec::new()),
            |(mut pass, mut frame), curr| {
                match curr.binding_rate {
                    MaterialBindingRate::PerFrame => frame.push(curr),
                    MaterialBindingRate::PerInstance => pass.push(curr),
                }
                (pass, frame)
            },
        );

        let per_pass = binding_sets(&per_pass);
        let per_frame = binding_sets(&per_frame);

        Ok(Self {
            pipeline,
            material_type: requirements.material_type,
            per_frame_binding_sets: per_frame.into(),
            per_instance_binding_sets: per_pass.into(),
        })
    }
}

pub struct MaterialInstance {
    pub material: Arc<Material>,
    /// Access: per_instance_descriptors[set]
    per_instance_descriptors: Box<[vk::DescriptorSet]>,
    /// Access: per_frame_descriptors[frame_index][set]
    per_frame_descriptors: Box<[Box<[vk::DescriptorSet]>]>,
    /// Access: per_instance_binded[set][binding]
    per_instance_binded: Box<[BindObjectSet]>,
    /// Access: per_frame_binded[frame_index][set - per_instance_set_count][binding]
    per_frame_binded: Box<[Box<[BindObjectSet]>]>,
    updated_bindings: BitSet,
}

impl MaterialInstance {
    pub fn new(
        material: Arc<Material>,
        descriptor_pool: vk::DescriptorPool,
        max_frames_in_flight: usize,
        device: &VulkanDevice,
    ) -> Result<Self> {
        let (per_instance_layouts, per_frame_layouts) = material
            .pipeline
            .set_layouts
            .split_at(material.per_instance_binding_sets.len());

        let per_frame_layouts = vec![per_frame_layouts.clone().to_vec(); max_frames_in_flight]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let layouts = [per_instance_layouts, &per_frame_layouts].concat();

        let descriptor_sets = {
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&layouts);
            unsafe { device.allocate_descriptor_sets(&allocate_info)? }
        };

        let (per_instance_sets, per_frame_sets) =
            descriptor_sets.split_at(material.per_instance_binding_sets.len());

        let per_instance_sets = per_instance_sets.to_owned().into_boxed_slice();
        let per_frame_sets = per_frame_sets
            .chunks(material.per_frame_binding_sets.len())
            .map(|chunk| chunk.to_owned().into_boxed_slice())
            .collect::<Box<[_]>>();

        let per_instance_binded = material
            .per_instance_binding_sets
            .iter()
            .map(|set| {
                let set = set
                    .iter()
                    .map(BindObject::from)
                    .collect::<Box<[BindObject]>>();
                BindObjectSet(set)
            })
            .collect::<Box<[_]>>();

        let per_frame_binded = material
            .per_frame_binding_sets
            .iter()
            .map(|set| {
                let set = set.iter().map(BindObject::from).collect::<Box<[_]>>();
                BindObjectSet(set)
            })
            .collect::<Box<[_]>>();

        let per_frame_binded =
            vec![per_frame_binded.clone(); max_frames_in_flight].into_boxed_slice();

        Ok(Self {
            material,
            per_frame_descriptors: per_frame_sets,
            per_instance_descriptors: per_instance_sets,
            per_frame_binded,
            per_instance_binded,
            updated_bindings: BitSet::new(),
        })
    }

    /// Sets a bind object for a per-instance descriptor set.
    /// *Important:* this call does not actually updates underlying descriptor set but only saves the binding for later
    ///
    /// # Parameters
    /// * set - descriptor set index as it appears in shader code
    /// * binding - binding index as it appears in shader code
    /// * object - an object to bind itself
    pub fn bind_to_instance(&mut self, set: usize, binding: usize, object: BindObject) {
        #[cfg(debug_assertions)]
        {
            let len = self.material.per_instance_binding_sets.len();
            if set >= len {
                let message = format!("Trying to bind to a per instance binding set, which is not a per instance binding set. Per instance binding set count: {len}, provided: {set}");
                core_critical!(message);
                panic!("{}", message);
            }

            let len = self.material.per_instance_binding_sets[set].len();
            if binding >= len {
                let msg = format!("Trying to bind to a per instance binding which is not present on the respective set. Set: {set}, Binding count: {len}, provided binding: {binding}");
                core_critical!(msg);
                panic!("{}", msg);
            }
        }

        let binded_object = &mut self.per_instance_binded[set][binding];
        if &object != binded_object {
            *binded_object = object;
            self.updated_bindings
                .add(make_binding_id(set as u32, binding as u32));
        }
    }

    /// Sets a bind object for a per-frame descriptor set.
    /// *Important:* this call does not actually updates underlying descriptor set but only saves the binding for later
    ///
    /// # Parameters
    /// * set - descriptor set index as it appears in shader code
    /// * binding - binding index as it appears in shader code
    /// * frame_index
    /// * object - an object to bind itself
    pub fn bind_to_frame(
        &mut self,
        set: usize,
        binding: usize,
        frame_index: usize,
        object: BindObject,
    ) {
        #[cfg(debug_assertions)]
        {
            let set_count = self.material.per_frame_binding_sets.len()
                + self.material.per_instance_binding_sets.len();

            if set >= set_count {
                let msg = format!("Trying to bind to a set which is not present in pipeline. Provided set: {}, last available set: {}", set, set_count - 1);
                core_critical!(msg);
                panic!("{}", msg);
            }
        }

        let frame_set = set.checked_sub(self.material.per_instance_binding_sets.len());

        #[cfg(debug_assertions)]
        {
            if frame_set.is_none() {
                let msg = format!("Trying to bind per-frame binding to a set which is not a per frame set. First per frame set: {}, provided set: {}", self.material.per_instance_binding_sets.len(), set);
                core_critical!(msg);
                panic!("{}", msg);
            }
        }

        let frame_set = frame_set.unwrap();

        #[cfg(debug_assertions)]
        {
            let binding_set_len = self.per_frame_binded[frame_index][frame_set].len();
            if binding >= binding_set_len {
                let msg = format!("Trying to bind to a binding index which is bigger than the last present on the binding set. Binding set length: {}, provided: {}", binding_set_len, binding);
                core_critical!(msg);
                panic!("{}", msg);
            }
        }

        let binded_object = &mut self.per_frame_binded[frame_index][frame_set][binding];
        if binded_object != &object {
            *binded_object = object;
            self.updated_bindings
                .add(make_binding_id(set as u32, binding as u32));
        }
    }

    pub fn update_descriptor_sets(&mut self, frame_index: usize, device: &VulkanDevice) {
        if self.updated_bindings.is_empty() {
            return;
        }

        let mut image_infos: Vec<vk::DescriptorImageInfo> = Vec::new();
        let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::new();

        let writes = (&self.updated_bindings)
            .iter()
            .map(|id| {
                let (set, binding) = get_set_binding_from_id(id);
                let set = set as usize;
                let binding = binding as usize;

                let is_frame_bind = set >= self.per_instance_descriptors.len();
                let set = if is_frame_bind {
                    set - self.per_instance_descriptors.len()
                } else {
                    set
                };
                let bind_object = if is_frame_bind {
                    let set = set - self.per_instance_descriptors.len();
                    &self.per_frame_binded[frame_index][set][binding]
                } else {
                    &self.per_instance_binded[set][binding]
                };

                let descriptor_set = if is_frame_bind {
                    self.per_frame_descriptors[frame_index][set]
                } else {
                    self.per_instance_descriptors[set]
                };

                let mut builder = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(binding as u32)
                    .descriptor_type(vk::DescriptorType::from(bind_object));

                match bind_object {
                    BindObject::InputAttachment(image) => {
                        const ERROR_MSG: &str = "Cannot bind an input attachment which is None";

                        #[cfg(debug_assertions)]
                        {
                            if image.is_none() {
                                core_critical!(ERROR_MSG);
                            }
                        }

                        let image_view = image.expect(ERROR_MSG);
                        let image_info = vk::DescriptorImageInfo::builder()
                            .image_view(image_view)
                            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                            .build();
                        image_infos.push(image_info);
                        builder = builder.image_info(&image_infos[image_infos.len() - 1..])
                    }
                    BindObject::UniformBuffer(ubo) => {
                        const ERROR_MSG: &str = "Cannot bind a uniform buffer which is None";

                        #[cfg(debug_assertions)]
                        {
                            if ubo.is_none() {
                                core_critical!(ERROR_MSG);
                            }
                        }

                        let ubo = ubo.expect(ERROR_MSG);
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(ubo)
                            .offset(0)
                            .range(vk::WHOLE_SIZE)
                            .build();

                        buffer_infos.push(buffer_info);
                        builder = builder.buffer_info(&buffer_infos[buffer_infos.len() - 1..])
                    }
                    BindObject::StorageBuffer(buffer) => {
                        const ERROR_MSG: &str = "Cannot bind a storage buffer which is None";

                        #[cfg(debug_assertions)]
                        {
                            if buffer.is_none() {
                                core_critical!(ERROR_MSG);
                            }
                        }

                        let ubo = buffer.expect(ERROR_MSG);
                        let buffer_info = vk::DescriptorBufferInfo::builder()
                            .buffer(ubo)
                            .offset(0)
                            .range(vk::WHOLE_SIZE)
                            .build();

                        buffer_infos.push(buffer_info);
                        builder = builder.buffer_info(&buffer_infos[buffer_infos.len() - 1..])
                    }
                }

                builder.build()
            })
            .collect::<Vec<_>>();

        unsafe { device.update_descriptor_sets(&writes, &[]) };

        self.updated_bindings.clear();
    }

    pub fn bind_pipeline(&self, cmd: vk::CommandBuffer, device: &VulkanDevice) {
        unsafe {
            device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.material.pipeline.handle,
            );
        }
    }

    pub fn bind_descriptors(
        &self,
        frame_index: usize,
        cmd: vk::CommandBuffer,
        device: &VulkanDevice,
    ) {
        let descriptor_sets = [
            self.per_instance_descriptors.clone(),
            self.per_frame_descriptors[frame_index].clone(),
        ]
        .concat();

        unsafe {
            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.material.pipeline.layout,
                0,
                &descriptor_sets,
                &[],
            )
        }
    }
}

fn make_binding_id(set: u32, binding: u32) -> u32 {
    (set << 16) + binding
}

fn get_set_binding_from_id(id: u32) -> (u32, u32) {
    let binding = id << 16 >> 16;
    let set = id >> 16;
    (set, binding)
}

#[cfg(test)]
mod tests {
    use specs::BitSet;

    use crate::material::get_set_binding_from_id;

    use super::make_binding_id;

    #[test]
    fn ids_are_retrievable() {
        let id0 = make_binding_id(0, 0);
        let id1 = make_binding_id(0, 1);
        let id2 = make_binding_id(0, 2);
        let id3 = make_binding_id(1, 0);
        let id4 = make_binding_id(1, 1);
        let id5 = make_binding_id(1, 2);
        let id6 = make_binding_id(1, 3);

        assert_eq!(get_set_binding_from_id(id0), (0, 0));
        assert_eq!(get_set_binding_from_id(id1), (0, 1));
        assert_eq!(get_set_binding_from_id(id2), (0, 2));
        assert_eq!(get_set_binding_from_id(id3), (1, 0));
        assert_eq!(get_set_binding_from_id(id4), (1, 1));
        assert_eq!(get_set_binding_from_id(id5), (1, 2));
        assert_eq!(get_set_binding_from_id(id6), (1, 3));
    }

    #[test]
    fn ids_are_insertable_to_bitset() {
        let mut bitset = BitSet::new();

        assert!(!bitset.add(make_binding_id(0, 0)));
        assert!(!bitset.add(make_binding_id(0, 1)));
        assert!(!bitset.add(make_binding_id(0, 2)));
        assert!(!bitset.add(make_binding_id(1, 0)));
        assert!(!bitset.add(make_binding_id(1, 1)));
        assert!(!bitset.add(make_binding_id(1, 2)));
        assert!(!bitset.add(make_binding_id(1, 3)));
    }

    #[test]
    fn ids_are_retrievable_from_bitset() {
        let mut bitset = BitSet::new();

        let bindings: Vec<(u32, u32)> =
            vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (1, 2), (1, 3)];

        println!("Adding bindings to bitset");

        for (set, binding) in &bindings {
            let id = make_binding_id(*set, *binding);
            println!("set: {set}, binding: {binding}, id: {id}");
            assert!(!bitset.add(make_binding_id(*set, *binding)));
        }

        println!("Retrieving bindings from bitset");

        for id in bitset {
            let (set, binding) = get_set_binding_from_id(id);
            println!("Got set({set}), binding({binding}) for id({id})");
            assert!(bindings.contains(&get_set_binding_from_id(id)))
        }
    }
}
