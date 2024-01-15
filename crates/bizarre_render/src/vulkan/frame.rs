use std::{
    collections::{BTreeMap, HashMap},
    mem::{align_of, size_of, size_of_val},
};

use anyhow::{bail, Result};
use ash::{util::Align, vk};

use nalgebra_glm::Mat4;

use crate::{
    mesh::Mesh,
    mesh_loader::MeshHandle,
    vertex::Vertex,
    vulkan_shaders::deferred,
    vulkan_utils::{buffer::create_buffer, framebuffer::create_framebuffer},
};

use super::image::VulkanImage;

const VBO_SIZE: usize = 10000 * size_of::<Vertex>();
const IBO_SIZE: usize = 100000 * size_of::<u32>();
const MODEL_LEN: usize = 100;

pub struct MeshRange {
    pub vbo_offset: i32,
    pub ibo_offset: u32,
    pub ibo_count: u32,
}

pub struct VulkanFrame {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub render_cmd: vk::CommandBuffer,
    pub render_cmd_fence: vk::Fence,
    pub setup_cmd: vk::CommandBuffer,
    pub setup_cmd_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,
    pub mesh_ranges: HashMap<MeshHandle, MeshRange>,

    pub deferred_set: vk::DescriptorSet,

    pub mesh_vbo: vk::Buffer,
    pub mesh_vbo_memory: vk::DeviceMemory,
    pub mesh_vbo_offset: usize,

    pub mesh_ibo: vk::Buffer,
    pub mesh_ibo_memory: vk::DeviceMemory,
    pub mesh_ibo_offset: usize,

    pub deferred_ubo: vk::Buffer,
    pub deferred_ubo_memory: vk::DeviceMemory,

    pub descriptor_pool: vk::DescriptorPool,

    pub color_image: VulkanImage,
    pub depth_image: VulkanImage,
    pub normals_image: VulkanImage,
}

pub struct VulkanFrameInfo<'a> {
    pub present_image: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub image_index: u32,
    pub cmd_pool: vk::CommandPool,
    pub descriptor_pool: vk::DescriptorPool,
    pub mem_props: &'a vk::PhysicalDeviceMemoryProperties,
    pub deferred_set_layout: vk::DescriptorSetLayout,
}

impl VulkanFrame {
    pub fn new(info: &VulkanFrameInfo, device: &ash::Device) -> Result<Self> {
        let (image_available_semaphore, render_finished_semaphore) = unsafe {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let ia_semaphore = device.create_semaphore(&semaphore_create_info, None)?;
            let rf_semaphore = device.create_semaphore(&semaphore_create_info, None)?;

            (ia_semaphore, rf_semaphore)
        };
        let extent = vk::Extent3D {
            width: info.extent.width,
            height: info.extent.height,
            depth: 1,
        };

        let depth_image = VulkanImage::new(
            extent,
            vk::Format::D32_SFLOAT,
            vk::ImageAspectFlags::DEPTH,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            info.mem_props,
            device,
        )?;

        let color_image = VulkanImage::new(
            extent,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageAspectFlags::COLOR,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            info.mem_props,
            device,
        )?;

        let normals_image = VulkanImage::new(
            extent,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ImageAspectFlags::COLOR,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            info.mem_props,
            device,
        )?;

        let framebuffer_attachments = [
            info.present_image,
            depth_image.view,
            color_image.view,
            normals_image.view,
        ];
        let framebuffer = create_framebuffer(
            &framebuffer_attachments,
            info.extent,
            info.render_pass,
            device,
        )?;

        let cmd_allocation_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(info.cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(2);

        let cmd_bufs = unsafe { device.allocate_command_buffers(&cmd_allocation_info)? };
        let render_cmd = cmd_bufs[0];
        let setup_cmd = cmd_bufs[1];

        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let render_cmd_fence = unsafe { device.create_fence(&fence_create_info, None)? };
        let setup_cmd_fence = unsafe { device.create_fence(&fence_create_info, None)? };

        let (mesh_vbo, mesh_vbo_memory) = create_buffer(
            VBO_SIZE,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.mem_props,
            device,
        )?;

        let (mesh_ibo, mesh_ibo_memory) = create_buffer(
            IBO_SIZE,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.mem_props,
            device,
        )?;

        let (deferred_ubo, deferred_ubo_memory) = create_buffer(
            size_of::<deferred::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            info.mem_props,
            device,
        )?;

        unsafe {
            let ptr = device
                .map_memory(
                    deferred_ubo_memory,
                    0,
                    size_of::<deferred::Ubo>() as u64,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<deferred::Ubo>();

            *ptr = deferred::Ubo {
                model: [Mat4::identity(); MODEL_LEN],
                view_projection: Mat4::identity(),
            };

            device.unmap_memory(deferred_ubo_memory);
        }

        let deferred_set_layouts = [info.deferred_set_layout];

        let deferred_set = {
            let allocation_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(info.descriptor_pool)
                .set_layouts(&deferred_set_layouts);

            unsafe { device.allocate_descriptor_sets(&allocation_info)?[0] }
        };

        let buffer_info = [vk::DescriptorBufferInfo::builder()
            .buffer(deferred_ubo)
            .range(size_of::<deferred::Ubo>() as vk::DeviceSize)
            .build()];

        let deferred_uniform_set_write = vk::WriteDescriptorSet::builder()
            .dst_set(deferred_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_info)
            .build();

        unsafe { device.update_descriptor_sets(&[deferred_uniform_set_write], &[]) };

        let vulkan_frame = Self {
            image_index: info.image_index,
            framebuffer,
            render_cmd,
            render_cmd_fence,
            setup_cmd,
            setup_cmd_fence,
            image_available_semaphore,
            render_finished_semaphore,
            mesh_ranges: HashMap::new(),
            mesh_vbo,
            mesh_vbo_memory,
            mesh_vbo_offset: 0,
            mesh_ibo,
            mesh_ibo_memory,
            mesh_ibo_offset: 0,
            deferred_set,
            deferred_ubo,
            deferred_ubo_memory,
            descriptor_pool: info.descriptor_pool,

            color_image,
            depth_image,
            normals_image,
        };

        Ok(vulkan_frame)
    }

    pub fn upload_meshes(&mut self, meshes: &[*const Mesh], device: &ash::Device) -> Result<()> {
        let (meshes, vbo_len, ibo_len) = meshes
            .iter()
            .map(|m| unsafe { &**m })
            .filter(|m| !self.mesh_ranges.contains_key(&m.id))
            .fold(
                (Vec::new(), 0, 0),
                |(mut meshes, mut vbo_len, mut ibo_len), mesh| {
                    meshes.push(mesh);
                    vbo_len += mesh.vertices.len();
                    ibo_len += mesh.indices.len();
                    (meshes, vbo_len, ibo_len)
                },
            );

        if (self.mesh_vbo_offset + vbo_len) * size_of::<Vertex>() > VBO_SIZE {
            bail!("Frame VBO overflow");
        }

        if (self.mesh_ibo_offset + ibo_len) * size_of::<u32>() > IBO_SIZE {
            bail!("Frame IBO overflow");
        }

        let mut vbo_data = Vec::with_capacity(vbo_len);
        let mut ibo_data = Vec::with_capacity(ibo_len);

        let mut vbo_tmp_offset = self.mesh_vbo_offset as i32;
        let mut ibo_tmp_offset = self.mesh_ibo_offset as u32;

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

        unsafe {
            let vbo_ptr = device.map_memory(
                self.mesh_vbo_memory,
                self.mesh_vbo_offset as u64 * size_of::<Vertex>() as u64,
                size_of_val(&vbo_data) as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            let ibo_ptr = device.map_memory(
                self.mesh_ibo_memory,
                self.mesh_ibo_offset as u64 * size_of::<u32>() as u64,
                size_of_val(&ibo_data) as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut vbo_align = Align::<Vertex>::new(
                vbo_ptr,
                align_of::<Vertex>() as u64,
                (size_of::<Vertex>() * vbo_data.len()) as u64,
            );

            let mut ibo_align = Align::<u32>::new(
                ibo_ptr,
                align_of::<Vertex>() as u64,
                (size_of::<u32>() * ibo_data.len()) as u64,
            );

            ibo_align.copy_from_slice(&ibo_data);
            vbo_align.copy_from_slice(&vbo_data);

            device.unmap_memory(self.mesh_vbo_memory);
            device.unmap_memory(self.mesh_ibo_memory);
        }

        Ok(())
    }

    pub fn destroy(&mut self, cmd_pool: vk::CommandPool, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            self.image_available_semaphore = vk::Semaphore::null();

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            device.destroy_framebuffer(self.framebuffer, None);
            self.framebuffer = vk::Framebuffer::null();

            let mut fences = [self.render_cmd_fence, self.setup_cmd_fence];

            for fence in fences.iter_mut() {
                device.destroy_fence(*fence, None);
                *fence = vk::Fence::null();
            }

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            device.destroy_semaphore(self.render_finished_semaphore, None);
            self.render_finished_semaphore = vk::Semaphore::null();

            let cmd_bufs = [self.render_cmd, self.setup_cmd];
            device.free_command_buffers(cmd_pool, &cmd_bufs);
            self.render_cmd = vk::CommandBuffer::null();

            let mut mems = [
                self.mesh_vbo_memory,
                self.mesh_ibo_memory,
                self.deferred_ubo_memory,
            ];

            for mem in mems.iter_mut() {
                device.free_memory(*mem, None);
                *mem = vk::DeviceMemory::null();
            }

            let mut bufs = [self.mesh_vbo, self.mesh_ibo, self.deferred_ubo];
            for buf in bufs.iter_mut() {
                device.destroy_buffer(*buf, None);
                *buf = vk::Buffer::null();
            }

            let mut images = [
                &mut self.color_image,
                &mut self.depth_image,
                &mut self.normals_image,
            ];

            for image in images.iter_mut() {
                image.destroy(device);
            }
        }
    }
}
