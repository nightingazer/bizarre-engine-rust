use std::{mem::size_of, sync::Arc};

use anyhow::Result;
use ash::{extensions::khr, vk};
use nalgebra_glm::Mat4;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{
    mesh_loader::get_mesh_loader,
    render_package::{MeshUpload, RenderPackage},
    vulkan::{
        device::VulkanDevice,
        frame::{VulkanFrame, VulkanFrameInfo},
        instance::VulkanInstance,
        pipeline::VulkanPipeline,
        render_pass::VulkanRenderPass,
        swapchain::VulkanSwapchain,
    },
    vulkan_shaders::deferred,
    vulkan_utils::pipeline::create_deferred_pipeline,
};

pub struct Renderer {
    instance: VulkanInstance,
    device: VulkanDevice,
    surface: vk::SurfaceKHR,
    surface_extent: vk::Extent2D,
    swapchain: VulkanSwapchain,
    viewport: vk::Viewport,
    render_pass: VulkanRenderPass,
    cmd_pool: vk::CommandPool,
    descriptor_pool: vk::DescriptorPool,
    frames: Vec<VulkanFrame>,
    max_frames_in_flight: usize,
    current_frame_index: usize,
    swapchain_images: Vec<vk::ImageView>,

    pending_mesh_uploads: Vec<Vec<MeshUpload>>,

    deferred_pipeline: VulkanPipeline,
}

impl Renderer {
    pub fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = unsafe { VulkanInstance::new(window.clone())? };
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let device = unsafe { VulkanDevice::new(&instance, surface)? };

        let window_extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let (swapchain, swapchain_images) =
            unsafe { VulkanSwapchain::new(&instance, &device, surface, &window_extent)? };

        let render_pass = VulkanRenderPass::new(swapchain.image_format, &window_extent, &device)?;

        let has_mirror_transform = swapchain
            .surface_capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::HORIZONTAL_MIRROR);
        let viewport = vk::Viewport {
            width: window_extent.width as f32,
            height: if has_mirror_transform {
                window_extent.height as f32
            } else {
                -(window_extent.height as f32)
            },
            min_depth: 0.0,
            max_depth: 1.0,
            x: 0.0,
            y: if has_mirror_transform {
                0.0
            } else {
                window_extent.height as f32
            },
        };

        let cmd_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .build();

            unsafe { device.handle.create_command_pool(&create_info, None)? }
        };

        let mem_props =
            unsafe { instance.get_physical_device_memory_properties(device.physical_device) };

        let deferred_pipeline = create_deferred_pipeline(&viewport, render_pass.handle, &device)?;

        let descriptor_pool = {
            let pool_sizes = [vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .build()];
            let create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(swapchain_images.len() as u32 * 4)
                .pool_sizes(&pool_sizes);
            unsafe { device.create_descriptor_pool(&create_info, None)? }
        };

        let frames = swapchain_images
            .iter()
            .enumerate()
            .map(|(i, present_image)| {
                VulkanFrame::new(
                    &VulkanFrameInfo {
                        extent: window_extent,
                        image_index: i as u32,
                        present_image: *present_image,
                        render_pass: render_pass.handle,
                        cmd_pool,
                        mem_props: &mem_props,
                        deferred_set_layout: deferred_pipeline.set_layout,
                        descriptor_pool,
                    },
                    &device,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        let max_frames_in_flight = swapchain_images.len();

        let system = Self {
            instance,
            device,
            surface,
            swapchain,
            swapchain_images,
            viewport,
            render_pass,
            cmd_pool,
            descriptor_pool,
            frames,
            current_frame_index: 0,
            max_frames_in_flight,
            surface_extent: window_extent,
            deferred_pipeline,
            pending_mesh_uploads: vec![Vec::new(); max_frames_in_flight],
        };

        Ok(system)
    }

    pub fn render(&mut self, render_package: &RenderPackage) -> Result<()> {
        let frame = &mut self.frames[self.current_frame_index];
        let (present_index, _) = unsafe {
            self.swapchain.swapchain_loader.acquire_next_image(
                *self.swapchain,
                u64::MAX,
                frame.image_available_semaphore,
                vk::Fence::null(),
            )?
        };

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];

        if !self.pending_mesh_uploads[self.current_frame_index].is_empty() {
            let mesh_loader = get_mesh_loader();
            let meshes = {
                self.pending_mesh_uploads[self.current_frame_index]
                    .drain(..)
                    .map(|m| mesh_loader.get(m.mesh))
                    .collect::<Result<Vec<_>>>()?
            };
            frame.upload_meshes(&meshes, &self.device)?;
        }

        if !render_package.mesh_uploads.is_empty() {
            let mesh_loader = get_mesh_loader();
            let meshes = {
                render_package
                    .mesh_uploads
                    .iter()
                    .map(|m| mesh_loader.get(m.mesh))
                    .collect::<Result<Vec<_>>>()?
            };
            frame.upload_meshes(&meshes, &self.device)?;

            for (i, pending) in self.pending_mesh_uploads.iter_mut().enumerate() {
                if i == self.current_frame_index {
                    continue;
                }

                pending.extend(render_package.mesh_uploads.clone());
            }
        }

        let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(frame.framebuffer)
            .render_area(self.surface_extent.into())
            .build();

        unsafe {
            let fences = [frame.render_cmd_fence];
            self.device.wait_for_fences(&fences, true, u64::MAX)?;

            self.device.reset_fences(&fences)?;

            self.device.reset_command_buffer(
                frame.render_cmd,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;

            self.device
                .begin_command_buffer(frame.render_cmd, &cmd_begin_info)?;

            self.device.cmd_begin_render_pass(
                frame.render_cmd,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            let viewports = [self.viewport];
            let scissors = [vk::Rect2D {
                extent: self.surface_extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            }];

            self.device
                .cmd_set_viewport(frame.render_cmd, 0, &viewports);
            self.device.cmd_set_scissor(frame.render_cmd, 0, &scissors);

            self.device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.deferred_pipeline.handle,
            );

            self.device
                .cmd_bind_vertex_buffers(frame.render_cmd, 0, &[frame.mesh_vbo], &[0]);

            self.device.cmd_bind_index_buffer(
                frame.render_cmd,
                frame.mesh_ibo,
                0,
                vk::IndexType::UINT32,
            );

            let deferred_uniform = deferred::Ubo {
                model: [Mat4::identity(); 100],
                view_projection: Mat4::identity(),
            };

            let deferred_uniform_data = unsafe {
                self.device
                    .map_memory(
                        frame.deferred_ubo_memory,
                        0,
                        size_of::<deferred::Ubo>() as vk::DeviceSize,
                        vk::MemoryMapFlags::empty(),
                    )?
                    .cast::<deferred::Ubo>()
            };

            *deferred_uniform_data = deferred_uniform;

            self.device.unmap_memory(frame.deferred_ubo_memory);

            self.device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.deferred_pipeline.layout,
                0,
                &[frame.deferred_set],
                &[],
            );

            self.device.cmd_end_render_pass(frame.render_cmd);

            self.device.end_command_buffer(frame.render_cmd)?;

            let wait_semaphores = [frame.image_available_semaphore];
            let signal_semaphores = [frame.render_finished_semaphore];
            let cmd_buffers = [frame.render_cmd];
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&cmd_buffers)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(&signal_semaphores)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .build();

            self.device.queue_submit(
                self.device.graphics_queue,
                &[submit_info],
                frame.render_cmd_fence,
            )?;

            let swapchains = [self.swapchain.handle];
            let indices = [present_index];
            let wait_semaphores = [frame.render_finished_semaphore];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices)
                .build();

            self.swapchain
                .swapchain_loader
                .queue_present(self.device.present_queue, &present_info)?;
        }

        self.current_frame_index = (self.current_frame_index + 1) % self.max_frames_in_flight;

        Ok(())
    }

    pub fn resize(&mut self, size: [u32; 2]) -> Result<()> {
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.frames
                .iter_mut()
                .for_each(|frame| frame.destroy(self.cmd_pool, &self.device.handle));

            self.swapchain_images
                .iter()
                .for_each(|&image| self.device.handle.destroy_image_view(image, None));

            self.device.handle.destroy_command_pool(self.cmd_pool, None);
            self.device
                .handle
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.deferred_pipeline.destroy(&self.device.handle);

            self.render_pass.destroy(&self.device.handle);

            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.handle, None);

            self.device.destroy_device(None);

            let surface_loader = khr::Surface::new(&self.instance.entry, &self.instance);

            surface_loader.destroy_surface(self.surface, None);

            #[cfg(feature = "vulkan_debug")]
            self.instance
                .debug_utils_loader
                .destroy_debug_utils_messenger(self.instance.debug_messenger, None);

            self.instance.destroy_instance(None);
        }
    }
}
