use std::collections::HashMap;

use anyhow::Result;
use ash::vk;
use bizarre_logger::core_debug;

use crate::{
    mesh::Mesh,
    mesh_loader::MeshHandle,
    vertex::MeshVertex,
    vulkan::{buffer::VulkanSliceBuffer, device::VulkanDevice},
    vulkan_shaders::{geometry_pass, lighting_pass},
};

const MAX_VERTICES: usize = 1_000_000;
const MAX_INDICES: usize = 3_500_000;
const MAX_TRANSFORMS: usize = 10_000;
const MAX_DIRECTIONAL_LIGHTS: usize = 100;

#[derive(Default)]
pub struct MeshRange {
    pub vbo_offset: i32,
    pub ibo_offset: u32,
    pub ibo_count: u32,
}

#[derive(Default)]
pub struct RenderScene {
    pub vbo: VulkanSliceBuffer<MeshVertex>,
    pub ibo: VulkanSliceBuffer<u32>,
    pub transforms: Box<[VulkanSliceBuffer<geometry_pass::TransformSSBO>]>,
    pub directional_lights: Box<[VulkanSliceBuffer<lighting_pass::DirectionalLightsSSBO>]>,

    pub mesh_ranges: HashMap<MeshHandle, MeshRange>,
    pub vbo_offset: i32,
    pub ibo_offset: u32,
}

impl RenderScene {
    pub fn new(max_frames_in_flight: usize, device: &VulkanDevice) -> Result<Self> {
        let vbo = VulkanSliceBuffer::new(
            MAX_VERTICES,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
        )?;

        let ibo = VulkanSliceBuffer::new(
            MAX_INDICES,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
        )?;

        let mut transforms = Vec::with_capacity(max_frames_in_flight);
        let mut directional_lights = Vec::with_capacity(max_frames_in_flight);

        for _ in 0..max_frames_in_flight {
            transforms.push(VulkanSliceBuffer::new(
                MAX_TRANSFORMS,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                device,
            )?);
            directional_lights.push(VulkanSliceBuffer::new(
                MAX_DIRECTIONAL_LIGHTS,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                device,
            )?);
        }

        let transforms = transforms.into_boxed_slice();
        let directional_lights = directional_lights.into_boxed_slice();

        Ok(Self {
            vbo,
            ibo,
            mesh_ranges: HashMap::default(),
            vbo_offset: 0,
            ibo_offset: 0,
            transforms,
            directional_lights,
        })
    }

    pub fn upload_meshes(&mut self, meshes: &[*const Mesh], device: &VulkanDevice) -> Result<()> {
        core_debug!("Uploading meshes to scene!");
        let (meshes, vbo_len, ibo_len) = meshes
            .iter()
            .map(|m| unsafe { &**m })
            .filter(|m| !self.mesh_ranges.contains_key(&m.id))
            .fold(
                (Vec::new(), 0, 0),
                |(mut meshes, mut vbo_len, mut ibo_len), mesh| {
                    core_debug!("Uploading mesh \"{}\" {:?}", mesh.name, mesh.id);
                    meshes.push(mesh);
                    vbo_len += mesh.vertices.len();
                    ibo_len += mesh.indices.len();
                    (meshes, vbo_len, ibo_len)
                },
            );

        let mut vbo_data = Vec::with_capacity(vbo_len);
        let mut ibo_data = Vec::with_capacity(ibo_len);

        let mut vbo_tmp_offset = self.vbo_offset;
        let mut ibo_tmp_offset = self.ibo_offset;

        for mesh in meshes.iter().cloned() {
            let range = MeshRange {
                vbo_offset: vbo_tmp_offset,
                ibo_offset: ibo_tmp_offset,
                ibo_count: mesh.indices.len() as u32,
            };
            self.mesh_ranges.insert(mesh.id, range);
            vbo_tmp_offset += mesh.vertices.len() as i32;
            ibo_tmp_offset += mesh.indices.len() as u32;
            vbo_data.extend_from_slice(&mesh.vertices);
            ibo_data.extend_from_slice(&mesh.indices);
        }

        let mut vbo_align =
            self.vbo
                .map_offset_count(self.vbo_offset as usize, vbo_data.len(), device)?;
        let mut ibo_align =
            self.ibo
                .map_offset_count(self.ibo_offset as usize, ibo_data.len(), device)?;

        vbo_align.copy_from_slice(&vbo_data);
        ibo_align.copy_from_slice(&ibo_data);

        self.vbo.unmap_memory(vbo_align, device);
        self.ibo.unmap_memory(ibo_align, device);

        self.vbo_offset = vbo_tmp_offset;
        self.ibo_offset = ibo_tmp_offset;

        Ok(())
    }

    pub fn upload_transforms(
        &self,
        transforms: &[geometry_pass::TransformSSBO],
        present_index: usize,
        device: &VulkanDevice,
    ) -> Result<()> {
        // TODO: fix the VulkanBuffer::map_range()
        unsafe {
            let mapped = device
                .map_memory(
                    self.transforms[present_index].memory,
                    0,
                    std::mem::size_of_val(transforms) as u64,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<geometry_pass::TransformSSBO>();

            let mapped = std::slice::from_raw_parts_mut(mapped, transforms.len());
            mapped.copy_from_slice(transforms);
            device.unmap_memory(self.transforms[present_index].memory);
        }
        Ok(())
    }

    pub fn upload_directional_lights(
        &self,
        lights: &[lighting_pass::DirectionalLightsSSBO],
        present_index: usize,
        device: &VulkanDevice,
    ) -> Result<()> {
        unsafe {
            let mapped = device
                .map_memory(
                    self.directional_lights[present_index].memory,
                    0,
                    std::mem::size_of_val(lights) as u64,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<lighting_pass::DirectionalLightsSSBO>();

            let mapped = std::slice::from_raw_parts_mut(mapped, lights.len());
            mapped.copy_from_slice(lights);
            device.unmap_memory(self.directional_lights[present_index].memory);
        }
        Ok(())
    }
}
