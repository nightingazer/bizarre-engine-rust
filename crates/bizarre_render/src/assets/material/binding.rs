use std::ops::{Deref, DerefMut};

use anyhow::Result;
use ash::vk;

use crate::{vulkan::device::VulkanDevice, vulkan_utils::shader::ShaderStage};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum BindingType {
    UniformBuffer = vk::DescriptorType::UNIFORM_BUFFER.as_raw(),
    StorageBuffer = vk::DescriptorType::STORAGE_BUFFER.as_raw(),
    InputAttachment = vk::DescriptorType::INPUT_ATTACHMENT.as_raw(),
}

impl From<BindingType> for vk::DescriptorType {
    fn from(value: BindingType) -> Self {
        vk::DescriptorType::from_raw(value as i32)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BindObject {
    UniformBuffer(Option<vk::Buffer>),
    InputAttachment(Option<vk::ImageView>),
    StorageBuffer(Option<vk::Buffer>),
}

impl From<&MaterialBinding> for BindObject {
    fn from(value: &MaterialBinding) -> Self {
        match value.binding_type {
            BindingType::InputAttachment => BindObject::InputAttachment(None),
            BindingType::UniformBuffer => BindObject::UniformBuffer(None),
            BindingType::StorageBuffer => BindObject::StorageBuffer(None),
        }
    }
}

impl From<&BindObject> for vk::DescriptorType {
    fn from(value: &BindObject) -> Self {
        match value {
            BindObject::InputAttachment(_) => vk::DescriptorType::INPUT_ATTACHMENT,
            BindObject::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
            BindObject::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindObjectSet(pub Box<[BindObject]>);

impl Deref for BindObjectSet {
    type Target = Box<[BindObject]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindObjectSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MaterialBindingRate {
    PerInstance,
    PerFrame,
}

/// Describes a binding from the shader perspective
#[derive(Debug, Clone)]
pub struct MaterialBinding {
    pub binding: u32,
    pub set: u32,
    pub shader_stage: ShaderStage,
    pub binding_type: BindingType,
    pub binding_rate: MaterialBindingRate,
}

impl From<&MaterialBinding> for vk::DescriptorSetLayoutBinding {
    fn from(value: &MaterialBinding) -> Self {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(value.binding)
            .descriptor_count(1)
            .stage_flags(value.shader_stage.into())
            .descriptor_type(value.binding_type.into())
            .build()
    }
}

#[derive(Clone, Debug)]
pub struct BindingSet(Box<[MaterialBinding]>);

impl From<Vec<MaterialBinding>> for BindingSet {
    fn from(value: Vec<MaterialBinding>) -> Self {
        Self(value.into())
    }
}

impl Deref for BindingSet {
    type Target = Box<[MaterialBinding]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BindingSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn binding_sets(bindings: &[MaterialBinding]) -> Vec<BindingSet> {
    if bindings.is_empty() {
        return vec![];
    }

    let (min_set, max_set) =
        bindings
            .iter()
            .map(|b| b.set)
            .fold((u32::MAX, u32::MIN), |acc, curr| {
                let (min, max) = acc;
                (curr.min(min), curr.max(max))
            });

    let length = (max_set - min_set) as usize + 1;

    bindings
        .iter()
        .cloned()
        .fold(vec![Vec::new(); length], |mut acc, curr| {
            acc[(curr.set - min_set) as usize].push(curr);
            acc
        })
        .into_iter()
        .map(BindingSet::from)
        .collect::<Vec<_>>()
}

pub fn bindings_into_layouts(
    bindings: &[MaterialBinding],
    device: &VulkanDevice,
) -> Result<Vec<vk::DescriptorSetLayout>> {
    if bindings.is_empty() {
        return Ok(vec![]);
    }
    let (min_set, max_set) =
        bindings
            .iter()
            .map(|b| b.set)
            .fold((u32::MAX, u32::MIN), |acc, curr| {
                let (min, max) = acc;
                (curr.min(min), curr.max(max))
            });

    let length = (max_set - min_set) as usize + 1;

    let layouts = bindings
        .iter()
        .fold(vec![Vec::new(); length], |mut acc, curr| {
            acc[(curr.set - min_set) as usize].push(vk::DescriptorSetLayoutBinding::from(curr));
            acc
        })
        .into_iter()
        .map(|bindings| {
            let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
            unsafe { Ok(device.create_descriptor_set_layout(&create_info, None)?) }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(layouts)
}
