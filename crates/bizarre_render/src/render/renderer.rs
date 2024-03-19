use std::{mem::size_of, sync::Arc};

use anyhow::{anyhow, bail, Result};
use ash::{
    extensions::khr,
    vk::{self, DeviceSize},
};
use bizarre_common::handle::Handle;
use bizarre_logger::{core_debug, core_error};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use thiserror::Error;

use crate::{
    material::{
        binding::BindObject,
        pipeline_features::{
            CullMode, PipelineFeatureFlags, PipelineFeatures, PolygonMode, PrimitiveTopology,
        },
        Material, MaterialInstance, MaterialType,
    },
    material_loader::{MaterialInstanceHandle, MaterialLoader},
    mesh_loader::{get_mesh_loader, MeshHandle},
    render_package::RenderPackage,
    scene::RenderScene,
    vertex::{PositionVertex, Vertex},
    vulkan::{
        device::VulkanDevice,
        frame::{VulkanFrame, VulkanFrameInfo},
        instance::VulkanInstance,
        pipeline::{VulkanPipeline, VulkanPipelineRequirements, VulkanPipelineStage},
        render_pass::VulkanRenderPass,
        swapchain::VulkanSwapchain,
    },
    vulkan_shaders::{
        geometry_pass,
        lighting_pass::{self, DirectionalLightsSSBO},
    },
    vulkan_utils::{buffer::create_buffer, shader::ShaderStage},
};

#[derive(Error, Debug)]
pub enum RenderException {
    #[error("Render skipped due to suboptimal swapchain state")]
    RenderSkippedOutOfDate,
}

pub struct Renderer {
    pub instance: VulkanInstance,
    pub device: VulkanDevice,
    descriptor_pool: vk::DescriptorPool,
    cmd_pool: vk::CommandPool,
    pub max_msaa: vk::SampleCountFlags,

    surface: vk::SurfaceKHR,
    surface_extent: vk::Extent2D,
    swapchain: VulkanSwapchain,
    viewport: vk::Viewport,
    pub render_pass: VulkanRenderPass,
    frames: Vec<VulkanFrame>,
    pub max_frames_in_flight: usize,

    screen_vbo: vk::Buffer,
    screen_vbo_memory: vk::DeviceMemory,

    ambient_pipeline: VulkanPipeline,
    directional_pipeline: VulkanPipeline,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Result<Self> {
        core_debug!("Constructing renderer!");
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

        let max_msaa = vk::SampleCountFlags::TYPE_2;

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
                vk::DescriptorPoolSize::builder()
                    .ty(vk::DescriptorType::STORAGE_BUFFER)
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

        let swapchain_image_views = swapchain.image_views.clone();

        let ambient_stages = [
            VulkanPipelineStage {
                path: "assets/shaders/ambient.vert".into(),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: "assets/shaders/ambient.frag".into(),
                stage: ShaderStage::Fragment,
            },
        ];

        let requirements = VulkanPipelineRequirements {
            attachment_count: 1,
            base_pipeline: None,
            bindings: &lighting_pass::ambient_material_bindings(),
            features: PipelineFeatures {
                culling: CullMode::None,
                flags: PipelineFeatureFlags::BLEND_ADD | PipelineFeatureFlags::DEPTH_TEST,
                polygon_mode: PolygonMode::Fill,
                primitive_topology: PrimitiveTopology::TriangleFan,
                ..Default::default()
            },
            material_type: MaterialType::Lighting,
            render_pass: render_pass.handle,
            sample_count: max_msaa,
            stage_definitions: &ambient_stages,
            vertex_attributes: PositionVertex::attribute_description(),
            vertex_bindings: PositionVertex::binding_description(),
        };

        let ambient_pipeline = VulkanPipeline::from_requirements(&requirements, &device)?;

        let directional_stages = [
            VulkanPipelineStage {
                path: "assets/shaders/directional.vert".into(),
                stage: ShaderStage::Vertex,
            },
            VulkanPipelineStage {
                path: "assets/shaders/directional.frag".into(),
                stage: ShaderStage::Fragment,
            },
        ];

        let requirements = VulkanPipelineRequirements {
            stage_definitions: &directional_stages,
            bindings: &lighting_pass::directional_material_bindings(),
            ..requirements
        };

        let directional_pipeline = VulkanPipeline::from_requirements(&requirements, &device)?;

        let frames = swapchain_image_views
            .iter()
            .enumerate()
            .map(|(i, _)| {
                VulkanFrame::new(
                    &VulkanFrameInfo {
                        extent: window_extent,
                        image_index: i as u32,
                        render_pass: render_pass.handle,
                        cmd_pool,
                        samples: max_msaa,
                        descriptor_pool,
                        ambient_set_layout: ambient_pipeline.set_layouts[0],
                        directional_set_layout: directional_pipeline.set_layouts[0],
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
            screen_vbo,
            screen_vbo_memory,
            ambient_pipeline,
            directional_pipeline,
        };

        Ok(system)
    }

    pub fn render(
        &mut self,
        render_package: &RenderPackage,
        render_scene: &mut RenderScene,
        material_loader: &MaterialLoader,
    ) -> Result<()> {
        let (present_index, mut suboptimal) = match self.acquire_image() {
            Ok((present_index, suboptimal)) => (present_index, suboptimal),
            Err(err) => match err.downcast_ref::<RenderException>() {
                Some(RenderException::RenderSkippedOutOfDate) => return Ok(()),
                None => return Err(err),
            },
        };

        self.prepare_scene(present_index, render_package, render_scene)?;
        self.render_to_image(present_index, render_package, render_scene, material_loader);

        suboptimal |= match self.present_image(present_index) {
            Ok(suboptimal) => suboptimal,
            Err(err) => return Err(err),
        };

        if suboptimal {
            self.recreate_swapchain()
        } else {
            Ok(())
        }
    }

    fn acquire_image(&mut self) -> Result<(usize, bool)> {
        let mut render_suboptimal = false;
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
                Ok((present_index, suboptimal)) => {
                    render_suboptimal = suboptimal;
                    present_index
                }
                Err(result) => match result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        core_debug!("Recreating swapchain: out of date");
                        self.device
                            .destroy_semaphore(image_available_semaphore, None);
                        self.recreate_swapchain()?;
                        return Err(RenderException::RenderSkippedOutOfDate.into());
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

        Ok((present_index, render_suboptimal))
    }

    fn prepare_scene(
        &mut self,
        present_index: usize,
        render_package: &RenderPackage,
        render_scene: &mut RenderScene,
    ) -> Result<()> {
        if !render_package.mesh_uploads.is_empty() {
            let mesh_loader = get_mesh_loader();
            let meshes = render_package
                .mesh_uploads
                .iter()
                .filter_map(|mu| mesh_loader.get(mu.mesh))
                .collect::<Vec<_>>();
            render_scene.upload_meshes(&meshes, &self.device)?;
        }
        if !render_package.directional_lights.is_empty() {
            let lights = render_package
                .directional_lights
                .iter()
                .map(DirectionalLightsSSBO::from)
                .collect::<Vec<_>>();

            render_scene.upload_directional_lights(&lights, present_index, &self.device)?;
        }
        Ok(())
    }

    fn render_to_image(
        &mut self,
        present_index: usize,
        render_package: &RenderPackage,
        render_scene: &RenderScene,
        material_loader: &MaterialLoader,
    ) -> Result<()> {
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

        let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(self.frames[present_index].framebuffer)
            .render_area(self.surface_extent.into())
            .build();

        struct UniqueDraw {
            mesh: MeshHandle,
            material_instance: MaterialInstanceHandle,
            instance_count: u32,
        }

        let (unique_draws, model_matrices) = {
            let draws = render_package.draw_submissions.clone();

            draws
                .chunk_by(|a, b| a.handle == b.handle && a.material_instance == b.material_instance)
                .map(|chunk| {
                    chunk.iter().fold(
                        (
                            UniqueDraw {
                                mesh: MeshHandle::null(),
                                material_instance: MaterialInstanceHandle::null(),
                                instance_count: 0,
                            },
                            Vec::new(),
                        ),
                        |mut acc, curr| {
                            if acc.0.mesh == Handle::null() {
                                acc.0.mesh = curr.handle;
                            }
                            if acc.0.material_instance == Handle::null() {
                                acc.0.material_instance = curr.material_instance;
                            }
                            acc.0.instance_count += 1;
                            acc.1.push(curr.model_matrix);
                            acc
                        },
                    )
                })
                .fold((Vec::new(), Vec::new()), |mut acc, curr| {
                    acc.0.push(curr.0);
                    acc.1.extend_from_slice(&curr.1);
                    acc
                })
        };

        let transforms = model_matrices
            .iter()
            .map(|t| geometry_pass::TransformSSBO::from(*t))
            .collect::<Vec<_>>();

        render_scene.upload_transforms(&transforms, present_index, &self.device)?;
        {
            let mut a = self.frames[present_index]
                .view_projection
                .map_memory(&self.device)?;
            a.view_projection = render_package.view_projection;
            self.frames[present_index]
                .view_projection
                .unmap_memory(a, &self.device);
        }

        unsafe {
            let frame = &mut self.frames[present_index];
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

            self.device.cmd_bind_vertex_buffers(
                frame.render_cmd,
                0,
                &[render_scene.vbo.buffer],
                &[0],
            );
            self.device.cmd_bind_index_buffer(
                frame.render_cmd,
                render_scene.ibo.buffer,
                0,
                vk::IndexType::UINT32,
            );

            let mut current_instance = 0;
            let mut last_bound_pipeline_id = vk::Pipeline::null();
            let mut last_bound_material_instance = MaterialInstanceHandle::null();
            for draw in unique_draws {
                if last_bound_material_instance != draw.material_instance {
                    let mut material_instance =
                        material_loader.get_instance_mut(draw.material_instance);
                    let pipeline_handle = material_instance.material.pipeline.handle;
                    if pipeline_handle != last_bound_pipeline_id {
                        material_instance.bind_pipeline(frame.render_cmd, &self.device);
                        last_bound_pipeline_id = pipeline_handle;
                    }
                    material_instance.bind_to_frame(
                        0,
                        1,
                        present_index,
                        BindObject::StorageBuffer(Some(
                            render_scene.transforms[present_index].buffer,
                        )),
                    );
                    material_instance.bind_to_frame(
                        0,
                        0,
                        present_index,
                        BindObject::UniformBuffer(Some(frame.view_projection.buffer)),
                    );
                    material_instance.update_descriptor_sets(present_index, &self.device);
                    material_instance.bind_descriptors(
                        present_index,
                        frame.render_cmd,
                        &self.device,
                    );
                    last_bound_material_instance = draw.material_instance;
                }

                match render_scene.mesh_ranges.get(&draw.mesh) {
                    Some(mesh_range) => self.device.cmd_draw_indexed(
                        frame.render_cmd,
                        mesh_range.ibo_count,
                        draw.instance_count,
                        mesh_range.ibo_offset,
                        mesh_range.vbo_offset,
                        current_instance,
                    ),
                    _ => panic!("Failed to get range for mesh {:?}", draw.mesh),
                };

                current_instance += draw.instance_count;
            }

            self.device
                .cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            // Lighting

            {
                let mut ambient_ubo = frame.ambient_ubo.map_memory(&self.device)?;
                ambient_ubo.ambient_color = render_package.ambient_color;
                frame.ambient_ubo.unmap_memory(ambient_ubo, &self.device);
            }

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

            {
                let lights_ssbo = [vk::DescriptorBufferInfo::builder()
                    .buffer(render_scene.directional_lights[present_index].buffer)
                    .range(vk::WHOLE_SIZE)
                    .build()];

                self.device.update_descriptor_sets(
                    &[vk::WriteDescriptorSet::builder()
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .dst_set(frame.directional_set)
                        .dst_binding(1)
                        .buffer_info(&lights_ssbo)
                        .build()],
                    &[],
                );
            }

            self.device.cmd_bind_descriptor_sets(
                frame.render_cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.directional_pipeline.layout,
                0,
                &[frame.directional_set],
                &[],
            );

            self.device.cmd_draw(
                frame.render_cmd,
                4,
                render_package.directional_lights.len() as u32,
                0,
                0,
            );

            self.device
                .cmd_next_subpass(frame.render_cmd, vk::SubpassContents::INLINE);

            // Translucent

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
                if self.max_msaa != vk::SampleCountFlags::TYPE_1 {
                    self.frames[present_index].resolve_image.image
                } else {
                    self.frames[present_index].output_image.image
                },
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
        }

        Ok(())
    }

    /// Present image to swapchain,
    /// Returns if the present was suboptimal
    fn present_image(&mut self, present_index: usize) -> Result<bool> {
        let swapchains = [self.swapchain.handle];
        let indices = [present_index as u32];
        let wait_semaphores = [self.frames[present_index].render_finished_semaphore];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices)
            .build();

        let present_result = unsafe {
            self.swapchain
                .swapchain_loader
                .queue_present(self.device.present_queue, &present_info)
        };

        match present_result {
            Err(err) => match err {
                vk::Result::ERROR_OUT_OF_DATE_KHR => {
                    self.recreate_swapchain()?;
                    Ok(false)
                }
                _ => Err(anyhow!("Renderer: Failed to present an image: {err}")),
            },
            _ => present_result.map_err(|err| anyhow!("{}", err)),
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

    pub fn resize(&mut self, size: [u32; 2]) {
        let extent = vk::Extent2D {
            width: size[0],
            height: size[1],
        };

        if self.surface_extent == extent {
            return;
        }

        self.surface_extent = extent;

        if let Err(err) = self.recreate_swapchain() {
            core_error!("Failed to resize, failed to recreate swapchain: {}", err)
        };
    }

    pub fn create_material(&self, requirements: &VulkanPipelineRequirements) -> Result<Material> {
        Material::new(requirements, &self.device)
    }

    pub fn create_material_instance(&self, material: Arc<Material>) -> Result<MaterialInstance> {
        MaterialInstance::new(
            material,
            self.descriptor_pool,
            self.max_frames_in_flight,
            &self.device,
        )
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        unsafe {
            self.device.device_wait_idle()?;
        }

        self.surface_extent = self.swapchain.recreate(self.surface, &self.device)?;

        self.viewport = create_viewport(self.surface_extent);

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
