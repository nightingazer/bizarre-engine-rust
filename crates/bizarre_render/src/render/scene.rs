use std::collections::HashMap;

use anyhow::Result;
use ash::vk;
use bizarre_logger::core_debug;

use crate::{
    mesh::Mesh,
    mesh_loader::MeshHandle,
    vertex::MeshVertex,
    vulkan::{
        buffer::{VulkanBuffer, VulkanSliceBuffer},
        device::VulkanDevice,
    },
};

const MAX_VERTICES: usize = 1_000_000;
const MAX_INDICES: usize = 3_500_000;

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

    pub mesh_ranges: HashMap<MeshHandle, MeshRange>,
    pub vbo_offset: i32,
    pub ibo_offset: u32,
}

impl RenderScene {
    pub fn new(device: &VulkanDevice) -> Result<Self> {
        let vbo = VulkanSliceBuffer::new(
            MAX_VERTICES,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
        )?;

        let a = vbo.map_range(.., device)?;
        vbo.unmap_memory(a, device);

        let ibo = VulkanSliceBuffer::new(
            MAX_VERTICES,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
        )?;

        let a = ibo.map_range(.., device)?;
        ibo.unmap_memory(a, device);

        Ok(Self {
            vbo,
            ibo,
            mesh_ranges: HashMap::default(),
            vbo_offset: 0,
            ibo_offset: 0,
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

        let mut vbo_tmp_offset = self.vbo_offset as i32;
        let mut ibo_tmp_offset = self.ibo_offset as u32;

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
}
