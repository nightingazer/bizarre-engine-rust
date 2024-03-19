use anyhow::Result;
use ash::vk;
use nalgebra_glm::Mat4;

use crate::{
    vulkan_shaders::{geometry_pass, lighting_pass},
    vulkan_utils::framebuffer::create_framebuffer,
};

use super::{buffer::VulkanBuffer, device::VulkanDevice, image::VulkanImage};

pub struct VulkanFrame {
    pub framebuffer: vk::Framebuffer,
    pub image_index: u32,
    pub render_cmd: vk::CommandBuffer,
    pub render_cmd_fence: vk::Fence,
    pub setup_cmd: vk::CommandBuffer,
    pub setup_cmd_fence: vk::Fence,
    pub image_available_semaphore: vk::Semaphore,
    pub render_finished_semaphore: vk::Semaphore,

    pub output_image: VulkanImage,
    pub color_image: VulkanImage,
    pub depth_image: VulkanImage,
    pub normals_image: VulkanImage,
    pub resolve_image: VulkanImage,

    // TODO: move that out of frame eventually
    pub view_projection: VulkanBuffer<geometry_pass::Ubo>,
    pub directional_set: vk::DescriptorSet,
    pub ambient_set: vk::DescriptorSet,
    pub ambient_ubo: VulkanBuffer<lighting_pass::AmbientUbo>,
}

pub struct VulkanFrameInfo {
    pub render_pass: vk::RenderPass,
    pub extent: vk::Extent2D,
    pub image_index: u32,
    pub cmd_pool: vk::CommandPool,
    pub samples: vk::SampleCountFlags,

    pub ambient_set_layout: vk::DescriptorSetLayout,
    pub directional_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
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

        let (ambient_set, directional_set) = {
            let layouts = [info.ambient_set_layout, info.directional_set_layout];
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(info.descriptor_pool)
                .set_layouts(&layouts);

            let sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };
            (sets[0], sets[1])
        };

        let ambient_ubo = VulkanBuffer::new(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            device,
        )?;

        let view_projection = VulkanBuffer::new(
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            device,
        )?;

        let color_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(color_image.view)
            .build()];

        let normals_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(normals_image.view)
            .build()];

        let directional_ubo_info = [vk::DescriptorBufferInfo::builder()
            .buffer(view_projection.buffer)
            .range(vk::WHOLE_SIZE)
            .build()];

        let ambient_ubo_info = [vk::DescriptorBufferInfo::builder()
            .buffer(ambient_ubo.buffer)
            .range(vk::WHOLE_SIZE)
            .build()];

        let set_writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(ambient_set)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&ambient_ubo_info)
                .dst_binding(0)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(directional_set)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&directional_ubo_info)
                .dst_binding(0)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(ambient_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_info)
                .dst_binding(1)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(ambient_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_info)
                .dst_binding(2)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(directional_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&color_info)
                .dst_binding(2)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(directional_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .image_info(&normals_info)
                .dst_binding(3)
                .build(),
        ];

        unsafe {
            device.update_descriptor_sets(&set_writes, &[]);
        }

        let vulkan_frame = Self {
            image_index: info.image_index,
            framebuffer,
            render_cmd,
            render_cmd_fence,
            setup_cmd,
            setup_cmd_fence,
            image_available_semaphore,
            render_finished_semaphore,
            output_image,
            color_image,
            depth_image,
            normals_image,
            resolve_image,
            ambient_set,
            ambient_ubo,
            directional_set,
            view_projection,
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

        self.update_sets(device);

        Ok(())
    }

    fn update_sets(&self, device: &VulkanDevice) {
        let colors_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.color_image.view)
            .build()];
        let normals_input_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.normals_image.view)
            .build()];

        let writes = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.ambient_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .dst_binding(1)
                .image_info(&colors_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.ambient_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .dst_binding(2)
                .image_info(&normals_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.directional_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .dst_binding(2)
                .image_info(&colors_input_info)
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.directional_set)
                .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
                .dst_binding(3)
                .image_info(&normals_input_info)
                .build(),
        ];

        unsafe {
            device.update_descriptor_sets(&writes, &[]);
        }
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
