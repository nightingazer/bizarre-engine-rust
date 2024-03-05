use std::mem::size_of;

use anyhow::{anyhow, bail, Result};
use ash::{
    extensions::khr,
    vk::{self, DeviceSize},
};
use bizarre_logger::core_debug;
use nalgebra_glm::{Mat4, Vec3};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{
    material::{
        pass::MaterialPassType,
        pipeline_features::{CullMode, PipelineFeatureFlags, PipelineFeatures, PrimitiveTopology},
    },
    mesh_loader::{get_mesh_loader, MeshHandle},
    render_package::{MeshUpload, RenderPackage},
    vertex::{MeshVertex, PositionVertex, Vertex},
    vulkan::{
        device::VulkanDevice,
        frame::{VulkanFrame, VulkanFrameInfo},
        instance::VulkanInstance,
        pipeline::{VulkanPipeline, VulkanPipelineRequirements, VulkanPipelineStage},
        render_pass::VulkanRenderPass,
        swapchain::VulkanSwapchain,
    },
    vulkan_shaders::{ambient, deferred, directional, floor},
    vulkan_utils::{buffer::create_buffer, shader::ShaderStage},
};

pub struct Renderer {
    instance: VulkanInstance,
    device: VulkanDevice,
    descriptor_pool: vk::DescriptorPool,
    cmd_pool: vk::CommandPool,
    max_msaa: vk::SampleCountFlags,

    surface: vk::SurfaceKHR,
    surface_extent: vk::Extent2D,
    swapchain: VulkanSwapchain,
    viewport: vk::Viewport,
    render_pass: VulkanRenderPass,
    frames: Vec<VulkanFrame>,
    max_frames_in_flight: usize,

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
        let instance = VulkanInstance::new(window)?;
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let device = VulkanDevice::new(&instance, surface)?;

        let max_msaa = vk::SampleCountFlags::TYPE_4;

        let window_extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let swapchain = VulkanSwapchain::new(&instance, &device, surface, &window_extent)?;

        let render_pass = VulkanRenderPass::new(max_msaa, &device)?;

        let viewport = create_viewport(window_extent);

        let descriptor_pool = {
            let pool_sizes = [
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(10)
                    .build(),
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::INPUT_ATTACHMENT)
                    .descriptor_count(10)
                    .build(),
            ];
            let create_info = vk::DescriptorPoolCreateInfo::builder()
                .max_sets(512)
                .pool_sizes(&pool_sizes);

            unsafe { device.create_descriptor_pool(&create_info, None)? }
        };

        let cmd_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.queue_family_index)
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .build();

            unsafe { device.handle.create_command_pool(&create_info, None)? }
        };

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
            stage_definitions: &[
                VulkanPipelineStage {
                    path: String::from("assets/shaders/deferred.vert"),
                    stage: ShaderStage::Vertex,
                },
                VulkanPipelineStage {
                    path: String::from("assets/shaders/deferred.frag"),
                    stage: ShaderStage::Fragment,
                },
            ],
            base_pipeline: None,
            vertex_attributes: MeshVertex::attribute_description(),
            vertex_bindings: MeshVertex::binding_description(),
            sample_count: max_msaa,
        };

        let deferred_pipeline = VulkanPipeline::from_requirements(&deferred_reqs, &device)?;

        let ambient_reqs = VulkanPipelineRequirements {
            attachment_count: 1,
            bindings: &ambient::material_bindings(),
            features: PipelineFeatures {
                flags: PipelineFeatureFlags::BLEND_ADD,
                primitive_topology: PrimitiveTopology::TriangleFan,
                ..deferred_reqs.features
            },
            pass_type: MaterialPassType::Lighting,
            stage_definitions: &[
                VulkanPipelineStage {
                    path: String::from("assets/shaders/ambient.vert"),
                    stage: ShaderStage::Vertex,
                },
                VulkanPipelineStage {
                    path: String::from("assets/shaders/ambient.frag"),
                    stage: ShaderStage::Fragment,
                },
            ],
            base_pipeline: Some(&deferred_pipeline),
            vertex_attributes: PositionVertex::attribute_description(),
            vertex_bindings: PositionVertex::binding_description(),
            ..deferred_reqs
        };

        let ambient_pipeline = VulkanPipeline::from_requirements(&ambient_reqs, &device)?;

        let directional_reqs = VulkanPipelineRequirements {
            bindings: &directional::material_bindings(),
            stage_definitions: &[
                VulkanPipelineStage {
                    path: String::from("assets/shaders/directional.vert"),
                    stage: ShaderStage::Vertex,
                },
                VulkanPipelineStage {
                    path: String::from("assets/shaders/directional.frag"),
                    stage: ShaderStage::Fragment,
                },
            ],
            base_pipeline: Some(&ambient_pipeline),
            ..ambient_reqs
        };

        let directional_pipeline = VulkanPipeline::from_requirements(&directional_reqs, &device)?;

        let floor_req = VulkanPipelineRequirements {
            bindings: &floor::material_bindings(),
            pass_type: MaterialPassType::Translucent,
            features: PipelineFeatures {
                culling: CullMode::None,
                primitive_topology: PrimitiveTopology::TriangleFan,
                flags: PipelineFeatureFlags::BLEND_COLOR_ALPHA | PipelineFeatureFlags::DEPTH_TEST,
                ..Default::default()
            },
            stage_definitions: &[
                VulkanPipelineStage {
                    path: String::from("assets/shaders/floor.vert"),
                    stage: ShaderStage::Vertex,
                },
                VulkanPipelineStage {
                    path: String::from("assets/shaders/floor.frag"),
                    stage: ShaderStage::Fragment,
                },
            ],
            vertex_attributes: <() as Vertex>::attribute_description(),
            vertex_bindings: <() as Vertex>::binding_description(),
            ..directional_reqs
        };

        let floor_pipeline = VulkanPipeline::from_requirements(&floor_req, &device)?;

        let swapchain_image_views = swapchain.image_views.clone();

        let frames = swapchain_image_views
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
                        samples: max_msaa,
                    },
                    &device,
                )
            })
            .collect::<Result<Vec<_>>>()?;

        let max_frames_in_flight = swapchain_image_views.len();

        let (screen_vbo, screen_vbo_memory) = create_buffer(
            size_of::<PositionVertex>() * 4,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE,
            &device,
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
            instance,
            device,
            max_msaa,
            surface,
            swapchain,
            viewport,
            render_pass,
            cmd_pool,
            descriptor_pool,
            frames,
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
        let image_available_semaphore = unsafe {
            let create_info = vk::SemaphoreCreateInfo::default();

            self.device
                .create_semaphore(&create_info, None)
                .map_err(|err| {
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
                    self.device
                        .destroy_semaphore(image_available_semaphore, None);
                    self.recreate_swapchain()?;
                    return Ok(());
                }
                Err(result) => match result {
                    vk::Result::SUBOPTIMAL_KHR | vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        core_debug!("Recreating swapchain: out of date");
                        self.device
                            .destroy_semaphore(image_available_semaphore, None);
                        self.recreate_swapchain()?;
                        return Ok(());
                    }
                    _ => bail!(result),
                },
            }
        };

        let present_index = present_index as usize;

        unsafe {
            self.device.wait_for_fences(
                &[self.frames[present_index].render_cmd_fence],
                true,
                u64::MAX,
            )?;
            self.device
                .destroy_semaphore(self.frames[present_index].image_available_semaphore, None);
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

            let deferred_uniform_mem = self
                .device
                .map_memory(
                    frame.deferred_ubo_memory,
                    0,
                    size_of::<deferred::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<deferred::Ubo>();

            let _ambient_uniform_mem = self
                .device
                .map_memory(
                    frame.ambient_ubo_memory,
                    0,
                    size_of::<ambient::Ubo>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?
                .cast::<ambient::Ubo>();

            let floor_ubo_mem = self
                .device
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

            self.device.unmap_memory(frame.deferred_ubo_memory);
            self.device.unmap_memory(frame.ambient_ubo_memory);
            self.device.unmap_memory(frame.floor_ubo_memory);

            self.device.cmd_bind_descriptor_sets(
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
                self.device.cmd_draw_indexed(
                    frame.render_cmd,
                    mesh_range.ibo_count,
                    *count,
                    mesh_range.ibo_offset,
                    mesh_range.vbo_offset,
                    current_instance,
                );

                current_instance += *count;
            }

            self.device
                .cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            self.device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.ambient_pipeline.handle,
            );

            self.device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.ambient_pipeline.layout,
                0,
                &[frame.ambient_set],
                &[],
            );

            self.device
                .cmd_bind_vertex_buffers(frame.render_cmd, 0, &[self.screen_vbo], &[0]);

            self.device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            self.device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.directional_pipeline.handle,
            );

            self.device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.directional_pipeline.layout,
                0,
                &[frame.directional_set],
                &[],
            );

            self.device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            self.device
                .cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            self.device.cmd_bind_pipeline(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.floor_pipeline.handle,
            );

            self.device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.floor_pipeline.layout,
                0,
                &[frame.floor_set],
                &[],
            );

            self.device.cmd_draw(frame.render_cmd, 4, 1, 0, 0);

            self.device.cmd_end_render_pass(frame.render_cmd);

            let image_barrier = vk::ImageMemoryBarrier2::builder()
                .image(self.swapchain.images[present_index])
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
                .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_array_layer: 0,
                    base_mip_level: 0,
                    layer_count: 1,
                    level_count: 1,
                })
                .build();

            let image_memory_barriers = &[image_barrier];
            let dependency_info = vk::DependencyInfo::builder()
                .image_memory_barriers(image_memory_barriers)
                .dependency_flags(vk::DependencyFlags::BY_REGION);

            self.device
                .cmd_pipeline_barrier2(frame.render_cmd, &dependency_info);

            self.blit_image(
                self.frames[present_index].render_cmd,
                self.frames[present_index].resolve_image.image,
                self.swapchain.images[present_index],
            );

            let frame = &mut self.frames[present_index];

            let image_barrier = vk::ImageMemoryBarrier2::builder()
                .image(self.swapchain.images[present_index])
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
                .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_array_layer: 0,
                    base_mip_level: 0,
                    layer_count: 1,
                    level_count: 1,
                })
                .build();

            let image_memory_barriers = &[image_barrier];
            let dependency_info =
                vk::DependencyInfo::builder().image_memory_barriers(image_memory_barriers);

            self.device
                .cmd_pipeline_barrier2(frame.render_cmd, &dependency_info);

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
                .queue_present(self.device.present_queue, &present_info);

            match present_result {
                Ok(true) => {
                    self.recreate_swapchain()?;
                    Ok(())
                }
                Err(err) => match err {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => {
                        self.recreate_swapchain()?;
                        Ok(())
                    }
                    _ => Err(anyhow!("Renderer: Failed to present an image: {err}")),
                },
                _ => Ok(()),
            }
        }
    }

    fn blit_image(
        &mut self,
        cmd: vk::CommandBuffer,
        src_image: vk::Image,
        present_image: vk::Image,
    ) {
        let blit_region = vk::ImageBlit2::builder()
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: self.surface_extent.width as i32,
                    y: self.surface_extent.height as i32,
                    z: 1,
                },
            ])
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D {
                    x: self.surface_extent.width as i32,
                    y: self.surface_extent.height as i32,
                    z: 1,
                },
            ])
            .src_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .dst_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();

        let blit_regions = [blit_region];

        let blit_image_info = vk::BlitImageInfo2::builder()
            .src_image(src_image)
            .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .dst_image(present_image)
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .filter(vk::Filter::LINEAR)
            .regions(&blit_regions);

        unsafe {
            self.device.cmd_blit_image2(cmd, &blit_image_info);
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

        self.recreate_swapchain()?;

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        unsafe {
            self.device.device_wait_idle()?;
        }

        self.swapchain.recreate(self.surface, &self.device)?;

        for frame in self.frames.iter_mut() {
            frame.recreate(
                self.surface_extent,
                *self.render_pass,
                self.max_msaa,
                &self.device,
            )?;
        }

        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.frames
                .iter_mut()
                .for_each(|frame| frame.destroy(self.cmd_pool, &self.device));

            self.device.destroy_command_pool(self.cmd_pool, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.ambient_pipeline.destroy(&self.device);
            self.deferred_pipeline.destroy(&self.device);
            self.directional_pipeline.destroy(&self.device);
            self.floor_pipeline.destroy(&self.device);

            self.render_pass.destroy(&self.device);

            self.swapchain
                .swapchain_loader
                .destroy_swapchain(self.swapchain.handle, None);

            self.device.free_memory(self.screen_vbo_memory, None);
            self.device.destroy_buffer(self.screen_vbo, None);

            let surface_loader = khr::Surface::new(&self.instance.entry, &self.instance);

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
