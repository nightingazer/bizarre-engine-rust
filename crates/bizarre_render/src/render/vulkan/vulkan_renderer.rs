use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use bizarre_logger::{core_debug, core_error, core_info, core_warn};
use nalgebra_glm::{look_at, perspective, vec2, vec3, Mat4};
use vulkano::{
    buffer::{subbuffer::BufferWriteGuard, Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo},
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo,
    },
    descriptor_set::{
        allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo},
        PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode},
        view::ImageView,
    },
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCallback, DebugUtilsMessengerCallbackData,
            DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateInfo,
    },
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{RenderPass, Subpass},
    swapchain::{
        self, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::GpuFuture,
    Validated, VulkanError, VulkanLibrary,
};
use vulkano_win::create_surface_from_winit;

use crate::{
    bitmap_font::BitmapFont, cube_map::CubeMap, render_math::AmbientLight,
    render_package::RenderPackage, renderer::Renderer, text::ScreenText,
};

use super::{
    frame::VulkanFrame,
    framebuffer::window_size_dependent_setup,
    pipeline::{
        create_deferred_pipeline, create_editor_grid_graphics_pipeline, create_lighting_pipeline,
        create_screen_text_pipeline, create_skybox_pipeline,
    },
    render_pass::create_render_pass,
    shaders::{
        ambient_frag, ambient_vert, deferred_frag, deferred_vert, directional_frag,
        directional_vert, floor_frag, floor_vert, skybox_frag, skybox_vert, text_frag, text_vert,
    },
    vertex::{VulkanColorNormalVertex, VulkanPosition2DVertex, VulkanVertex2D},
    vulkan_buffer::{create_ibo, create_ubo, create_vbo},
    vulkan_cube_map::VulkanCubeMap,
    vulkan_image::create_texture,
};

fn debug_messenger_callback(
    severity: DebugUtilsMessageSeverity,
    message_type: DebugUtilsMessageType,
    callback_data: DebugUtilsMessengerCallbackData<'_>,
) {
    match severity {
        DebugUtilsMessageSeverity::INFO => {
            core_info!("Vulkan({:?}): {}", message_type, callback_data.message);
        }
        DebugUtilsMessageSeverity::WARNING => {
            core_warn!("Vulkan({:?}): {}", message_type, callback_data.message);
        }
        DebugUtilsMessageSeverity::ERROR => {
            core_warn!("Vulkan({:?}): {}", message_type, callback_data.message);
        }
        _ => {
            core_debug!("Vulkan({:?}): {}", message_type, callback_data.message);
        }
    }
}

pub struct VulkanRenderer {
    instance: Arc<Instance>,
    surface: Arc<Surface>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    render_pass: Arc<RenderPass>,

    frames: Vec<VulkanFrame>,

    deferred_pipeline: Arc<GraphicsPipeline>,

    ambient_pipeline: Arc<GraphicsPipeline>,
    directional_pipeline: Arc<GraphicsPipeline>,
    skybox_pipeline: Arc<GraphicsPipeline>,
    floor_pipeline: Arc<GraphicsPipeline>,
    text_pipeline: Arc<GraphicsPipeline>,
    text_vbo: Subbuffer<[VulkanVertex2D]>,
    text_ibo: Subbuffer<[u32]>,
    text_obj: ScreenText,
    text_texture: Option<Arc<ImageView>>,

    color_buffer: Arc<ImageView>,
    normal_buffer: Arc<ImageView>,
    viewport: Viewport,

    commands: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
    image_index: u32,
    acquire_future: Option<SwapchainAcquireFuture>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    ambient_light: Subbuffer<AmbientLight>,
    fullscreen_quad: Subbuffer<[VulkanPosition2DVertex]>,
    view: Mat4,
    projection: Mat4,
    view_projection: Mat4,
    skybox_cube_map: Option<VulkanCubeMap>,
}

#[derive(Debug, thiserror::Error)]
enum RenderException {
    #[error("No vertices to render")]
    NothingToRender,
}

impl Renderer for VulkanRenderer {
    fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let instance = {
            let library = VulkanLibrary::new()?;

            // TODO: Replace that with Surface::required_extensions()
            let mut extensions = vulkano_win::required_extensions(&library);
            extensions.ext_debug_utils = true;

            let layers = vec!["VK_LAYER_KHRONOS_validation".to_owned()];

            Instance::new(
                library,
                InstanceCreateInfo {
                    enabled_extensions: extensions,
                    enabled_layers: layers,
                    max_api_version: Some(vulkano::Version::V1_1),
                    ..Default::default()
                },
            )?
        };

        let _debug_callback = unsafe {
            DebugUtilsMessenger::new(
                instance.clone(),
                DebugUtilsMessengerCreateInfo {
                    message_severity: DebugUtilsMessageSeverity::INFO
                        | DebugUtilsMessageSeverity::WARNING
                        | DebugUtilsMessageSeverity::ERROR,
                    message_type: DebugUtilsMessageType::GENERAL
                        | DebugUtilsMessageType::PERFORMANCE
                        | DebugUtilsMessageType::VALIDATION,
                    ..DebugUtilsMessengerCreateInfo::user_callback(
                        DebugUtilsMessengerCallback::new(debug_messenger_callback),
                    )
                },
            )
        };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let surface = create_surface_from_winit(window.clone(), instance.clone())?;

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &*surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .ok_or(anyhow!("No suitable physical device found"))?;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().ok_or(anyhow!("No queues found"))?;

        let (swapchain, images) = {
            let caps = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;

            let image_usage = caps.supported_usage_flags;
            let composite_alpha = caps
                .supported_composite_alpha
                .into_iter()
                .next()
                .ok_or(anyhow!("No supported alpha found"))?;

            let image_format = device
                .physical_device()
                .surface_formats(&surface, Default::default())?
                .iter()
                .next()
                .ok_or(anyhow!("No supported image formats found"))?
                .0;
            let image_extent: [u32; 2] = window.inner_size().into();

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_usage,
                    image_format,
                    image_extent,
                    composite_alpha,
                    ..Default::default()
                },
            )?
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo::default(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default(),
        ));

        let ambient_light = create_ubo(memory_allocator.clone(), AmbientLight::default())?;

        let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
        let projection = perspective(aspect, 90.0f32.to_radians(), 0.1, 250.0);
        let view = look_at(
            &vec3(0.0, 0.0, 10.0),
            &vec3(0.0, 0.0, 0.0),
            &vec3(0.0, 1.0, 0.0),
        );

        let view_projection = projection * view;

        let render_pass = create_render_pass(swapchain.clone(), device.clone())?;

        let deferred_pass = Subpass::from(render_pass.clone(), 0)
            .ok_or(anyhow!("Failed to create deferred pass from render pass"))?;
        let lighting_pass = Subpass::from(render_pass.clone(), 1)
            .ok_or(anyhow!("Failed to create lighting pass from render pass"))?;

        let deferred_pipeline = {
            let deferred_vert = deferred_vert::load(device.clone())?;
            let deferred_frag = deferred_frag::load(device.clone())?;

            create_deferred_pipeline::<VulkanColorNormalVertex>(
                deferred_vert,
                deferred_frag,
                deferred_pass.clone(),
                device.clone(),
                deferred_pass.num_color_attachments(),
            )?
        };

        let ambient_pipeline = {
            let ambient_vert = ambient_vert::load(device.clone())?;
            let ambient_frag = ambient_frag::load(device.clone())?;

            create_lighting_pipeline(
                ambient_vert,
                ambient_frag,
                lighting_pass.clone(),
                device.clone(),
                lighting_pass.num_color_attachments(),
            )?
        };

        let directional_pipeline = {
            let dir_vert = directional_vert::load(device.clone())?;
            let dir_frag = directional_frag::load(device.clone())?;

            create_lighting_pipeline(
                dir_vert,
                dir_frag,
                lighting_pass.clone(),
                device.clone(),
                lighting_pass.num_color_attachments(),
            )?
        };

        let skybox_pipeline = {
            let vert = skybox_vert::load(device.clone())?;
            let frag = skybox_frag::load(device.clone())?;

            let pass = Subpass::from(render_pass.clone(), 2).ok_or(anyhow!(
                "Failed to create editor background pass from render pass"
            ))?;

            create_skybox_pipeline(vert, frag, pass.clone(), device.clone())?
        };

        let floor_pipeline = {
            let floor_vert = floor_vert::load(device.clone())?;
            let floor_frag = floor_frag::load(device.clone())?;

            let floor_pass = Subpass::from(render_pass.clone(), 2)
                .ok_or(anyhow!("Failed to create floor pass from render pass"))?;

            create_editor_grid_graphics_pipeline(
                floor_vert,
                floor_frag,
                floor_pass.clone(),
                device.clone(),
                floor_pass.num_color_attachments(),
            )?
        };

        let text_pipeline = {
            let vert = text_vert::load(device.clone())?;
            let frag = text_frag::load(device.clone())?;

            let pass = Subpass::from(render_pass.clone(), 3)
                .ok_or(anyhow!("Failed to create text pass from render pass"))?;

            create_screen_text_pipeline(vert, frag, pass, device.clone())?
        };

        let text_obj = {
            let font = BitmapFont::new("assets/fonts/fira_sdf.fnt", "assets/fonts/fira_sdf.png")?;

            ScreenText {
                color: vec3(5.0, 4.0, 0.0),
                font,
                font_size: 100.0,
                position: vec2(0.0, 0.0),
                text: "Hello Vulkan!".into(),
            }
        };

        let mut viewport = Viewport {
            extent: [0.0, 0.0],
            offset: [0.0, 0.0],
            depth_range: 0.0..=1.0,
        };

        let (framebuffers, color_buffer, normal_buffer) = window_size_dependent_setup(
            &images,
            render_pass.clone(),
            &mut viewport,
            memory_allocator.clone(),
        )?;

        let screen_size = [viewport.extent[0] as u32, viewport.extent[1] as u32];
        let text_vbo = {
            let vbo = text_obj.vertex_buffer([800, 600]);
            create_vbo(
                memory_allocator.clone(),
                vbo.iter().cloned().map(|v| VulkanVertex2D::from(v)),
            )?
        };

        let text_ibo = create_ibo(memory_allocator.clone(), text_obj.index_buffer())?;

        let fullscreen_quad = VulkanPosition2DVertex::list();

        let fullscreen_quad = create_vbo(memory_allocator.clone(), fullscreen_quad.into_iter())?;

        let previous_frame_end = Some(Box::new(vulkano::sync::now(device.clone())) as Box<_>);

        let frames = framebuffers
            .iter()
            .enumerate()
            .map(|(i, framebuffer)| {
                let (deferred_set, deferred_ubo) = {
                    let ubo = deferred_vert::UBO {
                        view_projection: view_projection.into(),
                        model: [Mat4::default().into(); 100],
                    };

                    let ubo = create_ubo(memory_allocator.clone(), ubo)?;

                    let layout = deferred_pipeline.layout().set_layouts().first().unwrap();

                    let set = PersistentDescriptorSet::new(
                        &descriptor_set_allocator,
                        layout.clone(),
                        [WriteDescriptorSet::buffer(0, ubo.clone())],
                        [],
                    )?;

                    (set, ubo)
                };

                let frame = VulkanFrame {
                    deferred_set,
                    deferred_ubo,
                    frame_index: i as u32,
                    framebuffer: framebuffer.clone(),
                };

                Ok(frame)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            swapchain,

            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,

            render_pass,
            frames,

            deferred_pipeline,

            ambient_pipeline,
            directional_pipeline,
            skybox_pipeline,
            floor_pipeline,

            text_pipeline,
            text_vbo,
            text_ibo,
            text_obj,
            text_texture: None,

            color_buffer,
            normal_buffer,
            viewport,

            commands: None,
            image_index: 0,
            acquire_future: None,

            ambient_light,
            view_projection,
            view,
            projection,
            fullscreen_quad,

            previous_frame_end,
            skybox_cube_map: None,
        })
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, render_package: RenderPackage) -> Result<()> {
        let start_result = self.start_render(&render_package);
        let mut skip_render = false;
        match start_result {
            Err(e) => match e.downcast_ref::<RenderException>() {
                Some(RenderException::NothingToRender) => skip_render = true,
                _ => {
                    core_error!("Failed to start render: {}", e);
                    bail!(e);
                }
            },
            _ => (),
        };

        if !skip_render {
            self.deferred_render(&render_package)?;
        }

        self.commands.as_mut().unwrap().next_subpass(
            SubpassEndInfo::default(),
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )?;

        if !skip_render {
            self.ambient_render(&render_package)?;
            self.directional_render(&render_package)?;
        }

        self.commands.as_mut().unwrap().next_subpass(
            SubpassEndInfo::default(),
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )?;

        self.render_skybox()?;
        self.floor_render()?;

        self.commands.as_mut().unwrap().next_subpass(
            SubpassEndInfo::default(),
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )?;

        self.render_text()?;

        self.finish_render()?;

        Ok(())
    }

    fn resize(&mut self, _size: [u32; 2]) -> Result<()> {
        self.recreate_swapchain()?;
        Ok(())
    }
}

impl VulkanRenderer {
    fn recalculate_view_projection(&mut self) -> Result<()> {
        self.view_projection = self.projection * self.view;
        Ok(())
    }

    fn update_ambient_light(&mut self, ambient_light: AmbientLight) -> Result<()> {
        let mut write_guard = self.ambient_light.write()?;
        *write_guard = ambient_light;
        Ok(())
    }

    fn start_render(&mut self, render_package: &RenderPackage) -> Result<()> {
        self.previous_frame_end
            .as_mut()
            .take()
            .unwrap()
            .cleanup_finished();

        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(Validated::Error(e)) => match e {
                    VulkanError::OutOfDate => {
                        self.recreate_swapchain()?;
                        return Ok(());
                    }
                    _ => bail!(e),
                },
                Err(e) => bail!(e),
            };

        if suboptimal {
            self.recreate_swapchain()?;
        }

        let clear_values = vec![
            Some(render_package.clear_color.into()),
            Some([0.0, 0.0, 0.0, 0.0].into()),
            Some([0.0, 0.0, 0.0, 0.0].into()),
            Some(1.0.into()),
        ];

        if render_package.view_projection_was_updated {
            self.view = render_package.view.clone();
            self.projection = render_package.projection.clone();
            self.recalculate_view_projection();
        }

        if let Some(light) = &render_package.ambient_light {
            self.update_ambient_light(light.clone())?;
        }

        let mut commands = AutoCommandBufferBuilder::primary(
            &*self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        if self.skybox_cube_map.is_none() {
            self.skybox_cube_map = Some({
                let cube_map = CubeMap::new("assets/textures/sky_cubemap".into())
                    .expect("Failed to load cube map");
                VulkanCubeMap::new(
                    cube_map,
                    self.memory_allocator.clone(),
                    self.device.active_queue_family_indices(),
                    &mut commands,
                )
                .expect("Failed to convert cube map to vulkan format")
            })
        }

        if self.text_texture.is_none() {
            self.text_texture = Some({
                let texture = &self.text_obj.font.bitmap;
                create_texture(texture, self.memory_allocator.clone(), &mut commands)?
            })
        };

        let framebuffer = self.frames[image_index as usize].framebuffer.clone();

        commands
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values,
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .set_viewport(0, [self.viewport.clone()].into_iter().collect());

        self.image_index = image_index;
        self.acquire_future = Some(acquire_future);
        self.commands = Some(commands);

        if render_package.vertices.is_empty() && render_package.meshes.is_empty() {
            return Err(anyhow!(RenderException::NothingToRender));
        }

        Ok(())
    }

    fn deferred_render(&mut self, render_package: &RenderPackage) -> Result<()> {
        let commands = self.commands.as_mut().unwrap();

        {
            let mut ubo_guard: BufferWriteGuard<'_, deferred_vert::UBO> = self.frames
                [self.image_index as usize]
                .deferred_ubo
                .write()?;

            let mut model: [[[f32; 4]; 4]; 100] = [[[0.0; 4]; 4]; 100];

            for (i, mat) in render_package.model_matrices.into_iter().enumerate() {
                model[i] = mat.into()
            }

            *ubo_guard = deferred_vert::UBO {
                model,
                view_projection: self.view_projection.into(),
            };

            drop(ubo_guard);
        }

        let vertex_buffer = create_vbo(
            self.memory_allocator.clone(),
            render_package
                .vertices
                .iter()
                .map(|v| VulkanColorNormalVertex::from(v)),
        )?;
        let index_buffer = create_ibo(
            self.memory_allocator.clone(),
            render_package.indices.clone(),
        )?;

        let layout = self.deferred_pipeline.layout();
        let set = self.frames[self.image_index as usize].deferred_set.clone();

        commands
            .bind_pipeline_graphics(self.deferred_pipeline.clone())?
            .bind_vertex_buffers(0, vertex_buffer)?
            .bind_index_buffer(index_buffer)?
            .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)?;

        for submission in render_package.meshes.iter() {
            let first_index = submission.index_range.start;
            let last_index = submission.index_range.end;

            let push_constants = deferred_vert::Constants {
                model_offset: submission.model_matrix_offset,
            };

            commands
                .push_constants(layout.clone(), 0, push_constants)?
                .draw_indexed(last_index - first_index, 1, first_index, 0, 0)?;
        }

        Ok(())
    }

    fn ambient_render(&mut self, _render_package: &RenderPackage) -> Result<()> {
        let commands = self.commands.as_mut().unwrap();

        let ambient_layout = self.ambient_pipeline.layout();
        let set_layout = ambient_layout.set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.color_buffer.clone()),
                WriteDescriptorSet::buffer(1, self.ambient_light.clone()),
            ],
            [],
        )?;

        commands
            .bind_pipeline_graphics(self.ambient_pipeline.clone())?
            .bind_vertex_buffers(0, self.fullscreen_quad.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                ambient_layout.clone(),
                0,
                set.clone(),
            )?
            .draw(self.fullscreen_quad.len() as u32, 1, 0, 0)?;

        Ok(())
    }

    fn directional_render(&mut self, render_package: &RenderPackage) -> Result<()> {
        if render_package.directional_lights.is_empty() {
            return Ok(());
        }

        let commands = self.commands.as_mut().unwrap();

        let layout = self.directional_pipeline.layout();
        let set_layout = layout.set_layouts().get(0).unwrap();

        commands.bind_pipeline_graphics(self.directional_pipeline.clone());
        commands.bind_vertex_buffers(0, self.fullscreen_quad.clone());

        for light in render_package.directional_lights.iter() {
            let light = directional_frag::Directional_Data {
                color: light.color.into(),
                position: light.position.into(),
            };

            let buffer = create_ubo(self.memory_allocator.clone(), light.clone())?;

            let set = PersistentDescriptorSet::new(
                &self.descriptor_set_allocator,
                set_layout.clone(),
                [
                    WriteDescriptorSet::image_view(0, self.color_buffer.clone()),
                    WriteDescriptorSet::image_view(1, self.normal_buffer.clone()),
                    WriteDescriptorSet::buffer(2, buffer),
                ],
                [],
            )?;

            commands
                .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)?
                .draw(self.fullscreen_quad.len() as u32, 1, 0, 0)?;
        }

        Ok(())
    }

    fn render_skybox(&mut self) -> Result<()> {
        let commands = self.commands.as_mut().unwrap();

        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Nearest,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )?;

        let texture = self.skybox_cube_map.as_ref().unwrap();

        let vp = {
            let vp = skybox_vert::Uniforms {
                view: self.view.into(),
                projection: self.projection.into(),
            };

            create_ubo(self.memory_allocator.clone(), vp.clone())?
        };

        let layout = self.skybox_pipeline.layout();
        let set_layout = layout.set_layouts().first().unwrap();

        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [
                WriteDescriptorSet::buffer(0, vp),
                WriteDescriptorSet::image_view_sampler(1, texture.texture.clone(), sampler.clone()),
            ],
            [],
        )?;

        commands
            .bind_pipeline_graphics(self.skybox_pipeline.clone())?
            .bind_vertex_buffers(0, self.fullscreen_quad.clone())?
            .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)?
            .draw(self.fullscreen_quad.len() as u32, 1, 0, 0)
            .map_err(|e| anyhow!("Failed to draw editor background: {e}"))?;

        Ok(())
    }

    fn floor_render(&mut self) -> Result<()> {
        let mut commands = self.commands.as_mut().unwrap();

        let vp = {
            let vp = floor_vert::ViewProjectionUniforms {
                view: self.view.into(),
                projection: self.projection.into(),
            };

            create_ubo(self.memory_allocator.clone(), vp)?
        };

        let layout = self.floor_pipeline.layout();
        let set_layout = layout.set_layouts().first().unwrap();

        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [WriteDescriptorSet::buffer(0, vp)],
            [],
        )?;

        commands
            .bind_pipeline_graphics(self.floor_pipeline.clone())?
            .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)?
            .draw(6, 1, 0, 0)
            .map_err(|e| anyhow!("Failed to draw floor: {e}"))?;

        Ok(())
    }

    fn render_text(&mut self) -> Result<()> {
        let mut cmd = self.commands.as_mut().unwrap();

        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [SamplerAddressMode::ClampToEdge; 3],
                ..Default::default()
            },
        )?;

        let layout = self.text_pipeline.layout().clone();
        let set_layout = layout.set_layouts().first().unwrap();

        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                self.text_texture.as_ref().unwrap().clone(),
                sampler,
            )],
            [],
        )?;

        cmd.bind_pipeline_graphics(self.text_pipeline.clone())?
            .bind_vertex_buffers(0, self.text_vbo.clone())?
            .bind_index_buffer(self.text_ibo.clone())?
            .bind_descriptor_sets(PipelineBindPoint::Graphics, layout.clone(), 0, set)?
            .draw_indexed(self.text_ibo.len() as u32, 1, 0, 0, 0)?;

        Ok(())
    }

    fn finish_render(&mut self) -> Result<()> {
        let mut commands = self.commands.take().unwrap();
        commands.end_render_pass(SubpassEndInfo::default())?;
        let command_buffer = commands.build()?;

        let af = self.acquire_future.take().unwrap();

        let future = self
            .previous_frame_end
            .take()
            .ok_or(anyhow!("Failed to take prevuios frame end"))?
            .join(af)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(Validated::Error(e)) => match e {
                VulkanError::OutOfDate => {
                    self.recreate_swapchain()?;
                    self.previous_frame_end =
                        Some(Box::new(vulkano::sync::now(self.device.clone())));
                    return Ok(());
                }
                _ => return Err(anyhow!("Failed to flush future: {}", e)),
            },
            Err(e) => return Err(anyhow!("Failed to flush future: {}", e)),
        };

        self.commands = None;

        Ok(())
    }

    fn recreate_swapchain(&mut self) -> Result<()> {
        let size = self
            .surface
            .object()
            .unwrap()
            .downcast_ref::<winit::window::Window>()
            .unwrap()
            .inner_size()
            .into();

        let (swapchain, images) = self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: size,
            ..self.swapchain.create_info()
        })?;

        let (framebuffers, color_buffer, normal_buffer) = window_size_dependent_setup(
            &images,
            self.render_pass.clone(),
            &mut self.viewport,
            self.memory_allocator.clone(),
        )?;

        for (i, framebuffer) in framebuffers.iter().enumerate() {
            self.frames[i].framebuffer = framebuffer.clone();
        }

        self.swapchain = swapchain;
        self.color_buffer = color_buffer;
        self.normal_buffer = normal_buffer;

        Ok(())
    }
}
