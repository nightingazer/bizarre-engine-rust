use std::{f32, mem::size_of, sync::Arc};

use anyhow::{anyhow, bail, Result};
use ash::{
    extensions::khr,
    vk::{self, DeviceSize},
};
use bizarre_logger::core_debug;
use nalgebra_glm::{Mat4, Vec3};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{
    global_context::VULKAN_GLOBAL_CONTEXT,
    material::{
        pass::MaterialPassType,
        pipeline_features::{
            CullMode, PipelineFeatureFlags, PipelineFeatures, PolygonMode, PrimitiveTopology,
        },
    },
    mesh_loader::{get_mesh_loader, MeshHandle},
    render_package::{MeshUpload, RenderPackage},
    vertex::{MeshVertex, PositionVertex},
    vulkan::{
        device::VulkanDevice,
        frame::{VulkanFrame, VulkanFrameInfo},
        instance::VulkanInstance,
        pipeline::{VulkanPipeline, VulkanPipelineRequirements, VulkanPipelineStage},
        render_pass::VulkanRenderPass,
        swapchain::VulkanSwapchain,
    },
    vulkan_shaders::{ambient, deferred, floor},
    vulkan_utils::{
        buffer::create_buffer,
        pipeline::{
            create_ambient_light_pipeline, create_directional_pipeline, create_floor_pipeline,
        },
        shader::ShaderStage,
    },
};

pub struct Renderer {
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
    pending_view_projection: Vec<Option<Mat4>>,
    pending_view: Vec<Option<Mat4>>,
    pending_projection: Vec<Option<Mat4>>,
    pending_camera_forward: Vec<Option<Vec3>>,

    deferred_pipeline: VulkanPipeline,
    ambient_pipeline: VulkanPipeline,
    directional_pipeline: VulkanPipeline,
    floor_pipeline: VulkanPipeline,

    screen_vbo: vk::Buffer,
    screen_vbo_memory: vk::DeviceMemory,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        let instance = VULKAN_GLOBAL_CONTEXT.instance();
        let device = VULKAN_GLOBAL_CONTEXT.device();
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };

        let window_extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let swapchain = VulkanSwapchain::new(&instance, &device, surface, &window_extent)?;

        let swapchain_images = swapchain.image_views.clone();

        let render_pass = VulkanRenderPass::new(swapchain.image_format, &window_extent, &device)?;

        let viewport = create_viewport(window_extent);

        let cmd_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .build();

            unsafe { device.handle.create_command_pool(&create_info, None)? }
        };

        let deferred_stages = vec![
            VulkanPipelineStage {
                path: String::from("assets/shaders/deferred.vert"),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: String::from("assets/shaders/deferred.frag"),
                stage: ShaderStage::Fragment,
            },
        ];

        let deferred_reqs = VulkanPipelineRequirements {
            attachment_count: 2,
            bindings: &deferred::material_bindings(),
            features: PipelineFeatures {
                culling: CullMode::Back,
                flags: PipelineFeatureFlags::DEPTH_TEST | PipelineFeatureFlags::DEPTH_WRITE,
                ..Default::default()
            },
            pass_type: MaterialPassType::Geometry,
            render_pass: render_pass.handle,
            stage_definitions: &deferred_stages,
        };

        let deferred_pipeline = VulkanPipeline::from_requirements::<MeshVertex>(&deferred_reqs)?;

        let ambient_pipeline = create_ambient_light_pipeline(&viewport, render_pass.handle)?;
        let directional_pipeline = create_directional_pipeline(&viewport, render_pass.handle)?;
        let floor_pipeline = create_floor_pipeline(&viewport, render_pass.handle)?;

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
                        deferred_set_layout: deferred_pipeline.set_layout,
                        descriptor_pool,
                        ambient_set_layout: ambient_pipeline.set_layout,
                        directional_set_layout: directional_pipeline.set_layout,
                        floor_set_layout: floor_pipeline.set_layout,
                    },
                    &device,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        let max_frames_in_flight = swapchain_images.len();

        let (screen_vbo, screen_vbo_memory) = create_buffer(
            size_of::<PositionVertex>() * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        unsafe {
            let ptr = device
                .map_memory(
                    screen_vbo_memory,
                    0,
                    size_of::<PositionVertex>() as DeviceSize * 4,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast();

            *ptr = [
                PositionVertex {
                    position: [1.0, 1.0, 0.0].into(),
                },
                PositionVertex {
                    position: [-1.0, 1.0, 0.0].into(),
                },
                PositionVertex {
                    position: [-1.0, -1.0, 0.0].into(),
                },
                PositionVertex {
                    position: [1.0, -1.0, 0.0].into(),
                },
            ];

            device.unmap_memory(screen_vbo_memory);
        }

        let system = Self {
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
            pending_view_projection: vec![Some(Mat4::identity()); max_frames_in_flight],
            pending_view: vec![Some(Mat4::identity()); max_frames_in_flight],
            pending_projection: vec![Some(Mat4::identity()); max_frames_in_flight],
            pending_camera_forward: vec![Some(Vec3::zeros()); max_frames_in_flight],
            ambient_pipeline,
            directional_pipeline,
            floor_pipeline,
            screen_vbo,
            screen_vbo_memory,
        };

        Ok(system)
    }

    pub fn render(&mut self, render_package: &RenderPackage) -> Result<()> {
        let device = VULKAN_GLOBAL_CONTEXT.device();
        let image_available_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();

            device.create_semaphore(&create_info, None).map_err(|err| {
                anyhow!(
                    "Render: Failed to create image available semaphore: {:?}",
                    err
                )
            })?
        };

        let present_index = unsafe {
            let acquire_result = self.swapchain.swapchain_loader.acquire_next_image(
                *self.swapchain,
                u64::MAX,
                image_available_semaphore,
                vk::Fence::null(),
            );

            match acquire_result {
                Ok((present_index, false)) => present_index,
                Ok((_, true)) => {
                    core_debug!("Recreating swapchain: suboptimal");
                    unsafe {
                        device.destroy_semaphore(image_available_semaphore, None);
                    }
                    self.recreate_swapchain();
                    return Ok(());
                }
                Err(result) => match result {
                    vk::Result::SUBOPTIMAL_KHR | vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        core_debug!("Recreating swapchain: out of date");
                        unsafe {
                            device.destroy_semaphore(image_available_semaphore, None);
                        }
                        self.recreate_swapchain();
                        return Ok(());
                    }
                    _ => bail!(result),
                },
            }
        };

        let present_index = present_index as usize;

        unsafe {
            device.wait_for_fences(
                &[self.frames[present_index].render_cmd_fence],
                true,
                u64::MAX,
            )?;
            device.destroy_semaphore(self.frames[present_index].image_available_semaphore, None);
        }

        self.frames[present_index].image_available_semaphore = image_available_semaphore;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
            },
        ];

        self.update_frames(present_index, render_package)?;

        let frame = &mut self.frames[present_index];

        let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(frame.framebuffer)
            .render_area(self.surface_extent.into())
            .build();

        let (unique_handles, intance_counts, model_matrices) = {
            let mut sorted_draws = render_package.draw_submissions.clone();
            sorted_draws.sort_by(|a, b| a.handle.cmp(&b.handle));

            sorted_draws.iter().fold(
                (
                    Vec::<MeshHandle>::new(),
                    Vec::<u32>::new(),
                    Vec::<Mat4>::new(),
                ),
                |mut data, draw| {
                    if data.0.is_empty() || data.0.last().unwrap() != &draw.handle {
                        data.0.push(draw.handle);
                        data.1.push(1);
                    } else {
                        *data.1.last_mut().unwrap() += 1;
                    }
                    data.2.push(draw.model_matrix);
                    data
                },
            )
        };

        unsafe {
            let fences = [frame.render_cmd_fence];
            device.wait_for_fences(&fences, true, u64::MAX)?;

            device.reset_fences(&fences)?;

            device.reset_command_buffer(
                frame.render_cmd,
                vk::CommandBufferResetFlags::RELEASE_RESOURCES,
            )?;

            device.begin_command_buffer(frame.render_cmd, &cmd_begin_info)?;

            device.cmd_begin_render_pass(
                frame.render_cmd,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            let viewports = [self.viewport];
            let scissors = [vk::Rect2D {
                extent: self.surface_extent,
                offset: vk::Offset2D { x: 0, y: 0 },
            }];

            device.cmd_set_viewport(frame.render_cmd, 0, &viewports);
            device.cmd_set_scissor(frame.render_cmd, 0, &scissors);

            device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.deferred_pipeline.handle,
            );

            device.cmd_bind_vertex_buffers(frame.render_cmd, 0, &[frame.mesh_vbo], &[0]);

            device.cmd_bind_index_buffer(
                frame.render_cmd,
                frame.mesh_ibo,
                0,
                vk::IndexType::UINT32,
            );

            let deferred_uniform_mem = device
                .map_memory(
                    frame.deferred_ubo_memory,
                    0,
                    size_of::<deferred::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<deferred::Ubo>();

            let _ambient_uniform_mem = device
                .map_memory(
                    frame.ambient_ubo_memory,
                    0,
                    size_of::<ambient::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<ambient::Ubo>();

            let floor_ubo_mem = device
                .map_memory(
                    frame.floor_ubo_memory,
                    0,
                    size_of::<floor::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<floor::Ubo>();

            (*deferred_uniform_mem).model[..model_matrices.len()].copy_from_slice(&model_matrices);

            if let Some(view_projection) = self.pending_view_projection[present_index] {
                (*deferred_uniform_mem).view_projection = view_projection;
                self.pending_view_projection[present_index] = None;
            }

            if let Some(view) = self.pending_view[present_index] {
                (*floor_ubo_mem).view = view;
                self.pending_view[present_index] = None;
            }

            if let Some(projection) = self.pending_projection[present_index] {
                (*floor_ubo_mem).projection = projection;
                self.pending_projection[present_index] = None;
            }

            if let Some(view_projection) = render_package.view_projection {
                (*deferred_uniform_mem).view_projection = view_projection;
            }

            if let Some(view) = render_package.view {
                (*floor_ubo_mem).view = view;
            }
            if let Some(projection) = render_package.projection {
                (*floor_ubo_mem).projection = projection;
            }

            device.unmap_memory(frame.deferred_ubo_memory);
            device.unmap_memory(frame.ambient_ubo_memory);
            device.unmap_memory(frame.floor_ubo_memory);

            device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.deferred_pipeline.layout,
                0,
                &[frame.deferred_set],
                &[],
            );

            let mut current_instance = 0;
            for (handle, count) in unique_handles.iter().zip(intance_counts.iter()) {
                let mesh_range = frame.mesh_ranges.get(handle).unwrap();
                device.cmd_draw_indexed(
                    frame.render_cmd,
                    mesh_range.ibo_count,
                    *count,
                    mesh_range.ibo_offset,
                    mesh_range.vbo_offset,
                    current_instance,
                );

                current_instance += *count;
            }

            device.cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.ambient_pipeline.handle,
            );

            device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.ambient_pipeline.layout,
                0,
                &[frame.ambient_set],
                &[],
            );

            device.cmd_bind_vertex_buffers(frame.render_cmd, 0, &[self.screen_vbo], &[0]);

            device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.directional_pipeline.handle,
            );

            device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.directional_pipeline.layout,
                0,
                &[frame.directional_set],
                &[],
            );

            device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            device.cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.floor_pipeline.handle,
            );

            device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.floor_pipeline.layout,
                0,
                &[frame.floor_set],
                &[],
            );

            device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            device.cmd_end_render_pass(frame.render_cmd);

            device.end_command_buffer(frame.render_cmd)?;

            let wait_semaphores = [frame.image_available_semaphore];
            let signal_semaphores = [frame.render_finished_semaphore];
            let cmd_buffers = [frame.render_cmd];
            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&cmd_buffers)
                .wait_semaphores(&wait_semaphores)
                .signal_semaphores(&signal_semaphores)
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .build();

            device.queue_submit(
                device.graphics_queue,
                &[submit_info],
                frame.render_cmd_fence,
            )?;

            let swapchains = [self.swapchain.handle];
            let indices = [present_index as u32];
            let wait_semaphores = [frame.render_finished_semaphore];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices)
                .build();

            let present_result = self
                .swapchain
                .swapchain_loader
                .queue_present(device.present_queue, &present_info);

            match present_result {
                Ok(true) => {
                    self.recreate_swapchain();
                    Ok(())
                }
                Err(err) => match err {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                        self.recreate_swapchain();
                        Ok(())
                    }
                    _ => Err(anyhow!("Renderer: Failed to present an image: {err}")),
                },
                _ => Ok(()),
            }
        }
    }

    fn update_frames(
        &mut self,
        present_index: usize,
        render_package: &RenderPackage,
    ) -> Result<(), anyhow::Error> {
        let frame = &mut self.frames[present_index];

        if !self.pending_mesh_uploads[present_index].is_empty() {
            let mesh_loader = get_mesh_loader();
            let meshes = {
                self.pending_mesh_uploads[present_index]
                    .drain(..)
                    .map(|m| mesh_loader.get(m.mesh))
                    .collect::<Result<Vec<_>>>()?
            };
            frame.upload_meshes(&meshes)?;
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

            frame.upload_meshes(&meshes)?;

            for (i, pending) in self.pending_mesh_uploads.iter_mut().enumerate() {
                if i == present_index {
                    continue;
                }

                pending.extend(render_package.mesh_uploads.clone());
            }
        }
        Ok(
            if let Some(view_projection) = render_package.view_projection {
                for (i, pending) in self.pending_view_projection.iter_mut().enumerate() {
                    if i == present_index {
                        continue;
                    }

                    *pending = Some(view_projection);
                }

                if let Some(view) = render_package.view {
                    for (i, pending) in self.pending_view.iter_mut().enumerate() {
                        if i == present_index {
                            continue;
                        }

                        *pending = Some(view);
                    }
                }

                if let Some(projection) = render_package.projection {
                    for (i, pending) in self.pending_projection.iter_mut().enumerate() {
                        if i == present_index {
                            continue;
                        }

                        *pending = Some(projection);
                    }
                }

                if let Some(camera_forward) = render_package.camera_forward {
                    for (i, pending) in self.pending_camera_forward.iter_mut().enumerate() {
                        if i == present_index {
                            continue;
                        }

                        *pending = Some(camera_forward);
                    }
                }
            },
        )
    }

    pub fn resize(&mut self, size: [u32; 2]) -> Result<()> {
        self.surface_extent = vk::Extent2D {
            width: size[0],
            height: size[1],
        };

        self.viewport = create_viewport(self.surface_extent);

        self.recreate_swapchain();

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        unsafe {
            VULKAN_GLOBAL_CONTEXT.device().device_wait_idle();
        }

        self.swapchain
            .recreate(&self.surface_extent, self.surface)?;

        self.swapchain_images = self.swapchain.get_image_views()?;

        for (frame, image) in self.frames.iter_mut().zip(self.swapchain_images.clone()) {
            frame.recreate(self.surface_extent, image, *self.render_pass)?;
        }

        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            let device = VULKAN_GLOBAL_CONTEXT.device();
            let instance = VULKAN_GLOBAL_CONTEXT.instance();
            device.device_wait_idle().unwrap();

            self.frames
                .iter_mut()
                .for_each(|frame| frame.destroy(self.cmd_pool, &device.handle));

            self.swapchain_images
                .iter()
                .for_each(|&image| device.handle.destroy_image_view(image, None));

            device.handle.destroy_command_pool(self.cmd_pool, None);
            device
                .handle
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.ambient_pipeline.destroy();
            self.deferred_pipeline.destroy();
            self.directional_pipeline.destroy();
            self.floor_pipeline.destroy();

            self.render_pass.destroy(&device.handle);

            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.handle, None);

            device.free_memory(self.screen_vbo_memory, None);
            device.destroy_buffer(self.screen_vbo, None);

            let surface_loader = khr::Surface::new(&instance.entry, &instance);

            surface_loader.destroy_surface(self.surface, None);
        }
    }
}

fn create_viewport(window_extent: vk::Extent2D) -> vk::Viewport {
    vk::Viewport {
        width: window_extent.width as f32,
        height: -(window_extent.height as f32),
        min_depth: 0.0,
        max_depth: 1.0,
        x: 0.0,
        y: window_extent.height as f32,
    }
}
