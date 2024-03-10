use std::{
    collections::HashMap,
    mem::{align_of, size_of, size_of_val},
};

use anyhow::{bail, Result};
use ash::{util::Align, vk};

use nalgebra_glm::{vec3, Mat4};

use crate::{
    mesh::Mesh,
    mesh_loader::MeshHandle,
    vertex::MeshVertex,
    vulkan_shaders::{ambient, deferred, directional, floor},
    vulkan_utils::{buffer::create_buffer, framebuffer::create_framebuffer},
};

use super::{device::VulkanDevice, image::VulkanImage};

const VBO_SIZE: usize = 10000 * size_of::<MeshVertex>();
const IBO_SIZE: usize = 100000 * size_of::<u32>();
const MODEL_LEN: usize = 100;

pub struct VulkanFrame {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub render_cmd: vk::CommandBuffer,
    pub render_cmd_fence: vk::Fence,
    pub setup_cmd: vk::CommandBuffer,
    pub setup_cmd_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,

    pub deferred_set: vk::DescriptorSet,
    pub ambient_set: vk::DescriptorSet,
    pub directional_set: vk::DescriptorSet,
    pub floor_set: vk::DescriptorSet,

    pub deferred_ubo: vk::Buffer,
    pub deferred_ubo_memory: vk::DeviceMemory,

    pub ambient_ubo: vk::Buffer,
    pub ambient_ubo_memory: vk::DeviceMemory,

    pub directional_ubo: vk::Buffer,
    pub directional_ubo_memory: vk::DeviceMemory,

    pub floor_ubo: vk::Buffer,
    pub floor_ubo_memory: vk::DeviceMemory,

    pub descriptor_pool: vk::DescriptorPool,

    pub output_image: VulkanImage,
    pub color_image: VulkanImage,
    pub depth_image: VulkanImage,
    pub normals_image: VulkanImage,
    pub resolve_image: VulkanImage,
}

pub struct VulkanFrameInfo {
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub image_index: u32,
    pub cmd_pool: vk::CommandPool,
    pub descriptor_pool: vk::DescriptorPool,
    pub deferred_set_layout: vk::DescriptorSetLayout,
    pub ambient_set_layout: vk::DescriptorSetLayout,
    pub directional_set_layout: vk::DescriptorSetLayout,
    pub floor_set_layout: vk::DescriptorSetLayout,
    pub samples: vk::SampleCountFlags,
}

impl VulkanFrame {
    pub fn new(info: &VulkanFrameInfo, device: &VulkanDevice) -> Result<Self> {
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

        let (output_image, depth_image, color_image, normals_image, resolve_image) =
            create_frame_images(extent, info.samples, device)?;

        let framebuffer_attachments = [
            output_image.view,
            depth_image.view,
            color_image.view,
            normals_image.view,
            resolve_image.view,
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

        let (deferred_ubo, deferred_ubo_memory) = create_buffer(
            size_of::<deferred::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
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

        let (ambient_ubo, ambient_ubo_memory) = create_buffer(
            size_of::<ambient::Ubo>(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device,
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
            device,
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
            device,
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
            output_image,
            color_image,
            depth_image,
            normals_image,
            resolve_image,
        };

        Ok(vulkan_frame)
    }

    pub fn recreate(
        &mut self,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        samples: vk::SampleCountFlags,
        device: &VulkanDevice,
    ) -> Result<()> {
        self.destroy_images(device);

        let extent_3d = vk::Extent3D {
            depth: 1,
            height: extent.height,
            width: extent.width,
        };

        let (output, depth, color, normal, resolve) =
            create_frame_images(extent_3d, samples, device)?;

        unsafe {
            device.destroy_framebuffer(self.framebuffer, None);
        }

        let attachments = [
            output.view,
            depth.view,
            color.view,
            normal.view,
            resolve.view,
        ];

        self.framebuffer = create_framebuffer(&attachments, extent, render_pass, device)?;

        self.output_image = output;
        self.depth_image = depth;
        self.color_image = color;
        self.normals_image = normal;
        self.resolve_image = resolve;

        self.update_sets_with_images(device);

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
            &mut self.output_image,
            &mut self.color_image,
            &mut self.depth_image,
            &mut self.normals_image,
            &mut self.resolve_image,
        ];

        for image in images.iter_mut() {
            image.destroy(device);
        }
    }

    fn update_sets_with_images(&mut self, device: &VulkanDevice) {
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

        unsafe { device.update_descriptor_sets(&set_writes, &[]) };
    }
}

fn create_frame_images(
    extent: vk::Extent3D,
    samples: vk::SampleCountFlags,
    device: &VulkanDevice,
) -> Result<(
    VulkanImage,
    VulkanImage,
    VulkanImage,
    VulkanImage,
    VulkanImage,
)> {
    let output_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        samples,
        device,
    )?;
    let depth_image = VulkanImage::new(
        extent,
        vk::Format::D32_SFLOAT,
        vk::ImageAspectFlags::DEPTH,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        samples,
        device,
    )?;
    let color_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        samples,
        device,
    )?;
    let normals_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        samples,
        device,
    )?;
    let resolve_image = VulkanImage::new(
        extent,
        vk::Format::R16G16B16A16_SFLOAT,
        vk::ImageAspectFlags::COLOR,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::SampleCountFlags::TYPE_1,
        device,
    )?;
    Ok((
        output_image,
        depth_image,
        color_image,
        normals_image,
        resolve_image,
    ))
}
