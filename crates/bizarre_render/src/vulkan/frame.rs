use std::{
    collections::HashMap,
    mem::{align_of, size_of, size_of_val},
};

use anyhow::{bail, Result};
use ash::{util::Align, vk};

use nalgebra_glm::{vec3, Mat4};

use crate::{
    global_context::VULKAN_GLOBAL_CONTEXT,
    mesh::Mesh,
    mesh_loader::MeshHandle,
    vertex::MeshVertex,
    vulkan_shaders::{ambient, deferred, directional, floor},
    vulkan_utils::{buffer::create_buffer, framebuffer::create_framebuffer},
};

use super::image::VulkanImage;

const VBO_SIZE: usize = 10000 * size_of::<MeshVertex>();
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
    pub ambient_set: vk::DescriptorSet,
    pub directional_set: vk::DescriptorSet,
    pub floor_set: vk::DescriptorSet,

    pub mesh_vbo: vk::Buffer,
    pub mesh_vbo_memory: vk::DeviceMemory,
    pub mesh_vbo_offset: usize,

    pub mesh_ibo: vk::Buffer,
    pub mesh_ibo_memory: vk::DeviceMemory,
    pub mesh_ibo_offset: usize,

    pub deferred_ubo: vk::Buffer,
    pub deferred_ubo_memory: vk::DeviceMemory,

    pub ambient_ubo: vk::Buffer,
    pub ambient_ubo_memory: vk::DeviceMemory,

    pub directional_ubo: vk::Buffer,
    pub directional_ubo_memory: vk::DeviceMemory,

    pub floor_ubo: vk::Buffer,
    pub floor_ubo_memory: vk::DeviceMemory,

    pub descriptor_pool: vk::DescriptorPool,

    pub color_image: VulkanImage,
    pub depth_image: VulkanImage,
    pub normals_image: VulkanImage,
}

pub struct VulkanFrameInfo {
    pub present_image: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub image_index: u32,
    pub cmd_pool: vk::CommandPool,
    pub descriptor_pool: vk::DescriptorPool,
    pub deferred_set_layout: vk::DescriptorSetLayout,
    pub ambient_set_layout: vk::DescriptorSetLayout,
    pub directional_set_layout: vk::DescriptorSetLayout,
    pub floor_set_layout: vk::DescriptorSetLayout,
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

        let (depth_image, color_image, normals_image) = create_frame_images(extent)?;

        let framebuffer_attachments = [
            info.present_image,
            depth_image.view,
            color_image.view,
            normals_image.view,
        ];
        let framebuffer =
            create_framebuffer(&framebuffer_attachments, info.extent, info.render_pass)?;

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
        )?;

        let (mesh_ibo, mesh_ibo_memory) = create_buffer(
            IBO_SIZE,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let (deferred_ubo, deferred_ubo_memory) = create_buffer(
            size_of::<deferred::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
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

        let (ambient_ubo, ambient_ubo_memory) = create_buffer(
            size_of::<ambient::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let ptr = device
                .map_memory(
                    ambient_ubo_memory,
                    0,
                    size_of::<ambient::Ubo>() as u64,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast();

            *ptr = ambient::Ubo {
                color: [0.1, 0.15, 0.23],
            };

            device.unmap_memory(ambient_ubo_memory);
        }

        let (directional_ubo, directional_ubo_memory) = create_buffer(
            size_of::<directional::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let ptr = device
                .map_memory(
                    directional_ubo_memory,
                    0,
                    size_of::<directional::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast();

            *ptr = directional::Ubo {
                color: vec3(0.9, 0.75, 0.55),
                direction: vec3(1.0, 1.0, 1.0).normalize(),
                ..Default::default()
            };

            device.unmap_memory(directional_ubo_memory);
        }

        let (floor_ubo, floor_ubo_memory) = create_buffer(
            std::mem::size_of::<floor::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            let ptr = device
                .map_memory(
                    floor_ubo_memory,
                    0,
                    std::mem::size_of::<floor::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast();

            *ptr = floor::Ubo {
                view: Mat4::identity(),
                projection: Mat4::identity(),
            };

            device.unmap_memory(floor_ubo_memory);
        }

        let set_layouts = [
            info.deferred_set_layout,
            info.ambient_set_layout,
            info.directional_set_layout,
            info.floor_set_layout,
        ];

        let descriptor_sets = {
            let allocation_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(info.descriptor_pool)
                .set_layouts(&set_layouts);

            unsafe { device.allocate_descriptor_sets(&allocation_info)? }
        };

        let [deferred_set, ambient_set, directional_set, floor_set] = descriptor_sets.as_slice()
        else {
            bail!("Descriptor set allocation failed")
        };

        let deferred_ubo_info = [vk::DescriptorBufferInfo::builder()
            .buffer(deferred_ubo)
            .range(size_of::<deferred::Ubo>() as vk::DeviceSize)
            .build()];

        let ambient_ubo_info = [vk::DescriptorBufferInfo::builder()
            .buffer(ambient_ubo)
            .range(size_of::<ambient::Ubo>() as vk::DeviceSize)
            .build()];

        let directional_ubo_info = [vk::DescriptorBufferInfo::builder()
            .buffer(directional_ubo)
            .range(vk::WHOLE_SIZE)
            .build()];

        let floor_ubo_info = [vk::DescriptorBufferInfo::builder()
            .range(vk::WHOLE_SIZE)
            .buffer(floor_ubo)
            .build()];

        let color_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(color_image.view)
            .build()];

        let normals_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(normals_image.view)
            .build()];

        let set_writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(*deferred_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&deferred_ubo_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*ambient_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&ambient_ubo_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*ambient_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*ambient_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*directional_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&directional_ubo_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*directional_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*directional_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(*floor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&floor_ubo_info)
                .build(),
        ];

        unsafe { device.update_descriptor_sets(&set_writes, &[]) };

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
            deferred_set: *deferred_set,
            deferred_ubo,
            deferred_ubo_memory,
            descriptor_pool: info.descriptor_pool,
            ambient_set: *ambient_set,
            ambient_ubo,
            ambient_ubo_memory,
            directional_set: *directional_set,
            directional_ubo,
            directional_ubo_memory,
            floor_set: *floor_set,
            floor_ubo,
            floor_ubo_memory,
            color_image,
            depth_image,
            normals_image,
        };

        Ok(vulkan_frame)
    }

    pub fn upload_meshes(&mut self, meshes: &[*const Mesh]) -> Result<()> {
        let device = VULKAN_GLOBAL_CONTEXT.device();
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

        if (self.mesh_vbo_offset + vbo_len) * size_of::<MeshVertex>() > VBO_SIZE {
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
                self.mesh_vbo_offset as u64 * size_of::<MeshVertex>() as u64,
                size_of_val(&vbo_data) as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            let ibo_ptr = device.map_memory(
                self.mesh_ibo_memory,
                self.mesh_ibo_offset as u64 * size_of::<u32>() as u64,
                size_of_val(&ibo_data) as u64,
                vk::MemoryMapFlags::empty(),
            )?;

            let mut vbo_align = Align::<MeshVertex>::new(
                vbo_ptr,
                align_of::<MeshVertex>() as u64,
                (size_of::<MeshVertex>() * vbo_data.len()) as u64,
            );

            let mut ibo_align = Align::<u32>::new(
                ibo_ptr,
                align_of::<MeshVertex>() as u64,
                (size_of::<u32>() * ibo_data.len()) as u64,
            );

            ibo_align.copy_from_slice(&ibo_data);
            vbo_align.copy_from_slice(&vbo_data);

            device.unmap_memory(self.mesh_vbo_memory);
            device.unmap_memory(self.mesh_ibo_memory);
        }

        Ok(())
    }

    pub fn recreate(
        &mut self,
        extent: vk::Extent2D,
        present_image: vk::ImageView,
        render_pass: vk::RenderPass,
    ) -> Result<()> {
        let device = VULKAN_GLOBAL_CONTEXT.device();

        self.destroy_images(device);

        let extent_3d = vk::Extent3D {
            depth: 1,
            height: extent.height,
            width: extent.width,
        };

        let (depth, color, normal) = create_frame_images(extent_3d)?;

        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
        }

        let attachments = [present_image, depth.view, color.view, normal.view];

        self.framebuffer = create_framebuffer(&attachments, extent, render_pass)?;

        self.depth_image = depth;
        self.color_image = color;
        self.normals_image = normal;

        self.update_sets_with_images();

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
                self.ambient_ubo_memory,
                self.directional_ubo_memory,
                self.floor_ubo_memory,
            ];

            for mem in mems.iter_mut() {
                device.free_memory(*mem, None);
                *mem = vk::DeviceMemory::null();
            }

            let mut bufs = [
                self.mesh_vbo,
                self.mesh_ibo,
                self.deferred_ubo,
                self.ambient_ubo,
                self.directional_ubo,
                self.floor_ubo,
            ];
            for buf in bufs.iter_mut() {
                device.destroy_buffer(*buf, None);
                *buf = vk::Buffer::null();
            }

            self.destroy_images(device);
        }
    }

    fn destroy_images(&mut self, device: &ash::Device) {
        let mut images = [
            &mut self.color_image,
            &mut self.depth_image,
            &mut self.normals_image,
        ];

        for image in images.iter_mut() {
            image.destroy(device);
        }
    }

    fn update_sets_with_images(&mut self) -> Result<()> {
        let color_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.color_image.view)
            .build()];

        let normals_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.normals_image.view)
            .build()];

        let set_writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.ambient_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.ambient_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.directional_set)
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.directional_set)
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_input_info)
                .build(),
        ];

        unsafe {
            VULKAN_GLOBAL_CONTEXT
                .device()
                .update_descriptor_sets(&set_writes, &[])
        };

        Ok(())
    }
}

fn create_frame_images(
    extent: vk::Extent3D,
) -> Result<(VulkanImage, VulkanImage, VulkanImage), anyhow::Error> {
    let depth_image = VulkanImage::new(
        extent,
        vk::Format::D32_SFLOAT,
        vk::ImageAspectFlags::DEPTH,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let color_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let normals_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    Ok((depth_image, color_image, normals_image))
}
