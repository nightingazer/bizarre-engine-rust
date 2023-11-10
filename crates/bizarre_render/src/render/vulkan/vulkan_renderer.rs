use std::sync::Arc;

use anyhow::{anyhow, Result};
use bizarre_logger::{core_debug, core_error, core_info, core_warn};
use nalgebra_glm::{half_pi, look_at, perspective, vec3};
use vulkano::instance::debug::{
    DebugUtilsMessageSeverity, DebugUtilsMessenger, DebugUtilsMessengerCreateInfo, Message,
};
use vulkano::pipeline::Pipeline;
use vulkano::render_pass::Subpass;
use vulkano::swapchain::{SurfaceTransform, SurfaceTransforms, Swapchain};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator},
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, PipelineBindPoint},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{AcquireError, SwapchainCreateInfo, SwapchainCreationError, SwapchainPresentInfo},
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::create_surface_from_handle_ref;

use crate::{render_math::ViewProjection, render_package::RenderPackage, renderer::Renderer};

use super::pipeline::create_graphics_pipeline;
use super::render_pass::create_render_pass;
use super::shaders::{
    ambient_frag, ambient_vert, deferred_frag, deferred_vert, directional_frag, directional_vert,
};
use super::vertex::VulkanVertexData;

pub struct VulkanRenderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    viewport: Viewport,
    queue: Arc<Queue>,

    recreate_swapchain: bool,
    surface_size: [u32; 2],

    framebuffers: Vec<Arc<Framebuffer>>,
    color_buffer: Arc<ImageView<AttachmentImage>>,
    normals_buffer: Arc<ImageView<AttachmentImage>>,
    render_pass: Arc<RenderPass>,

    deferred_pipeline: Arc<GraphicsPipeline>,
    ambient_pipeline: Arc<GraphicsPipeline>,
    directional_pipeline: Arc<GraphicsPipeline>,

    previous_frame_end: Option<Box<dyn GpuFuture>>,

    cmd_buffer_allocator: StandardCommandBufferAllocator,

    _debug_messenger: DebugUtilsMessenger,
}

fn debug_messenger_user_callback(msg: &Message) {
    match msg.severity {
        DebugUtilsMessageSeverity::ERROR => {
            core_error!("Vulkan: {}", msg.description);
        }
        DebugUtilsMessageSeverity::WARNING => {
            core_warn!("Vulkan: {}", msg.description);
        }
        DebugUtilsMessageSeverity::INFO => {
            core_info!("Vulkan: {}", msg.description);
        }
        DebugUtilsMessageSeverity::VERBOSE => {
            core_debug!("Vulkan: {}", msg.description);
        }
        _ => {
            core_debug!("Vulkan: {}", msg.description);
        }
    }
}

impl Renderer for VulkanRenderer {
    fn new(window: &winit::window::Window) -> Result<Self>
    where
        Self: Sized,
    {
        let instance = {
            let library = VulkanLibrary::new()?;
            let mut extensions = vulkano_win::required_extensions(&library);
            extensions.ext_debug_utils = true;
            Instance::new(
                library,
                InstanceCreateInfo {
                    enabled_extensions: extensions,
                    enumerate_portability: true,
                    max_api_version: Some(vulkano::Version::V1_1),
                    ..Default::default()
                },
            )?
        };

        let surface = unsafe { create_surface_from_handle_ref(window, instance.clone())? };

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .ok_or(anyhow!("Failed to find suitable gpu"))?;

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index: queue_family_index as u32,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;

        let queue = queues.next().ok_or(anyhow!("Failed to get queue"))?;

        let (swapchain, images) = {
            let caps = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())?;
            let usage = caps.supported_usage_flags;
            let alpha = caps
                .supported_composite_alpha
                .into_iter()
                .next()
                .ok_or(anyhow!("Failed to find supported composite alpha"))?;

            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())?[0]
                    .0,
            );

            let image_extent: [u32; 2] = window.inner_size().into();

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: caps.min_image_count,
                    image_format,
                    image_extent,
                    image_usage: usage,
                    composite_alpha: alpha,
                    ..Default::default()
                },
            )?
        };

        let cmd_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let render_pass = create_render_pass(swapchain.clone(), device.clone())?;

        let allocator: StandardMemoryAllocator =
            StandardMemoryAllocator::new_default(device.clone());

        let (framebuffers, color_buffer, normals_buffer) =
            window_size_dependent_setup(&images, render_pass.clone(), &mut viewport, &allocator)?;

        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

        let deferred_subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let lighting_subpass = Subpass::from(render_pass.clone(), 1).unwrap();

        let deferred_pipeline = {
            let deferred_vert = deferred_vert::load(device.clone())?;
            let deferred_frag = deferred_frag::load(device.clone())?;
            create_graphics_pipeline(
                deferred_vert,
                deferred_frag,
                deferred_subpass,
                device.clone(),
                None,
                None,
            )?
        };

        let ambient_pipeline = {
            let ambient_vert = ambient_vert::load(device.clone())?;
            let ambient_frag = ambient_frag::load(device.clone())?;
            create_graphics_pipeline(
                ambient_vert,
                ambient_frag,
                lighting_subpass.clone(),
                device.clone(),
                Some(true),
                Some(lighting_subpass.num_color_attachments()),
            )?
        };

        let directional_pipeline = {
            let directional_vert = directional_vert::load(device.clone())?;
            let directional_frag = directional_frag::load(device.clone())?;
            create_graphics_pipeline(
                directional_vert,
                directional_frag,
                lighting_subpass.clone(),
                device.clone(),
                Some(true),
                Some(lighting_subpass.num_color_attachments()),
            )?
        };

        Ok(Self {
            _debug_messenger: unsafe {
                DebugUtilsMessenger::new(
                    instance.clone(),
                    DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                        debug_messenger_user_callback(msg)
                    })),
                )?
            },

            _instance: instance,
            device,
            swapchain,
            viewport,
            queue,

            recreate_swapchain: false,
            surface_size: window.inner_size().into(),

            deferred_pipeline,
            ambient_pipeline,
            directional_pipeline,

            framebuffers,
            color_buffer,
            normals_buffer,
            render_pass,

            previous_frame_end,

            cmd_buffer_allocator,
        })
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn render(&mut self, render_package: RenderPackage) -> Result<()> {
        self.previous_frame_end
            .as_mut()
            .take()
            .unwrap()
            .cleanup_finished();

        if self.recreate_swapchain {
            let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
                image_extent: self.surface_size,
                ..self.swapchain.create_info()
            }) {
                Ok(r) => r,
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => {
                    core_debug!("Failed to recreate swapchain: Image extent not supported");
                    return Ok(());
                }
                Err(e) => return Err(anyhow!("Failed to recreate swapchain: {}", e)),
            };

            self.swapchain = new_swapchain;

            let allocator = StandardMemoryAllocator::new_default(self.device.clone());
            let (framebuffers, color_buffer, normals_buffer) = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.viewport,
                &allocator,
            )?;
            self.framebuffers = framebuffers;
            self.color_buffer = color_buffer;
            self.normals_buffer = normals_buffer;
            self.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => return Err(anyhow!("Failed to acquire next image: {}", e)),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let clear_values = vec![
            Some(render_package.clear_color.into()),
            Some([0.0, 0.0, 0.0, 1.0].into()),
            Some([0.0, 0.0, 0.0, 1.0].into()),
            Some(1.0.into()),
        ];

        let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.cmd_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let vertices: Vec<VulkanVertexData> = render_package
            .vertices
            .into_iter()
            .map(|v| v.into())
            .collect();

        let memory_allocator = StandardMemoryAllocator::new_default(self.device.clone());

        let vertex_buffer = Buffer::from_iter(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            vertices,
        )?;

        let index_buffer = {
            let indices = render_package.indices.clone();
            Buffer::from_iter(
                &memory_allocator,
                BufferCreateInfo {
                    usage: BufferUsage::INDEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                indices,
            )?
        };

        let mvp_data = {
            let mut mvp = ViewProjection::default();

            let aspect_ratio = self.surface_size[0] as f32 / self.surface_size[1] as f32;
            mvp.projection = perspective(aspect_ratio, half_pi(), 0.01, 100.0);
            mvp.view = look_at(
                &render_package.camera_position,
                &vec3(0.0, 0.0, 0.0),
                &vec3(0.0, 1.0, 0.0),
            );

            crate::render::vulkan::shaders::deferred_vert::MVP_Data {
                projection: mvp.projection.into(),
                view: mvp.view.into(),
            }
        };

        let mvp_uniform = Buffer::from_data(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            mvp_data,
        )?;

        let ambient_light = crate::render::vulkan::shaders::ambient_frag::Ambient_Data {
            color: render_package.ambient_light.color,
            intencity: render_package.ambient_light.intensity,
        };

        let ambient_uniform = Buffer::from_data(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            ambient_light,
        )?;

        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(self.device.clone());

        let deferred_layout = self
            .deferred_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap();
        let deferred_set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            deferred_layout.clone(),
            [WriteDescriptorSet::buffer(0, mvp_uniform.clone())],
        )?;

        let ambient_layout = self.ambient_pipeline.layout().set_layouts().get(0).unwrap();
        let ambient_set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            ambient_layout.clone(),
            [
                WriteDescriptorSet::image_view(0, self.color_buffer.clone()),
                WriteDescriptorSet::buffer(1, mvp_uniform.clone()),
                WriteDescriptorSet::buffer(2, ambient_uniform.clone()),
            ],
        )?;

        cmd_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values,
                    ..RenderPassBeginInfo::framebuffer(
                        self.framebuffers[image_index as usize].clone(),
                    )
                },
                SubpassContents::Inline,
            )?
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.deferred_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.deferred_pipeline.layout().clone(),
                0,
                deferred_set.clone(),
            )
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .bind_index_buffer(index_buffer.clone())
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?
            .next_subpass(SubpassContents::Inline)?
            .bind_pipeline_graphics(self.ambient_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.ambient_pipeline.layout().clone(),
                0,
                ambient_set.clone(),
            )
            .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?
            .bind_pipeline_graphics(self.directional_pipeline.clone());

        for light in render_package.directional_lights {
            let directional_light =
                crate::render::vulkan::shaders::directional_frag::Directional_Data {
                    position: light.position.into(),
                    color: light.color,
                };

            let directional_uniform = Buffer::from_data(
                &memory_allocator,
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    usage: MemoryUsage::Upload,
                    ..Default::default()
                },
                directional_light,
            )?;

            let directional_layout = self
                .directional_pipeline
                .layout()
                .set_layouts()
                .get(0)
                .unwrap();
            let directional_set = PersistentDescriptorSet::new(
                &descriptor_set_allocator,
                directional_layout.clone(),
                [
                    WriteDescriptorSet::image_view(0, self.color_buffer.clone()),
                    WriteDescriptorSet::image_view(1, self.normals_buffer.clone()),
                    WriteDescriptorSet::buffer(2, mvp_uniform.clone()),
                    WriteDescriptorSet::buffer(3, directional_uniform.clone()),
                ],
            )?;

            cmd_buffer_builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.directional_pipeline.layout().clone(),
                    0,
                    directional_set.clone(),
                )
                .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)?;
        }

        cmd_buffer_builder.end_render_pass()?;

        let cmd_buffer = cmd_buffer_builder.build()?;

        let future = self
            .previous_frame_end
            .take()
            .ok_or(anyhow!("Failed to take previous frame end"))?
            .join(acquire_future)
            .then_execute(self.queue.clone(), cmd_buffer)?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
                Ok(())
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to flush future: {}", e)),
        }
    }

    fn resize(&mut self, size: [u32; 2]) -> Result<()> {
        self.recreate_swapchain = true;
        self.surface_size = size;

        Ok(())
    }
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    allocator: &StandardMemoryAllocator,
) -> Result<(
    Vec<Arc<Framebuffer>>,
    Arc<ImageView<AttachmentImage>>,
    Arc<ImageView<AttachmentImage>>,
)> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, -(dimensions[1] as f32)];
    viewport.origin = [0.0, dimensions[1] as f32];

    let color_buffer = ImageView::new_default(AttachmentImage::transient_input_attachment(
        allocator,
        dimensions,
        Format::A2B10G10R10_UNORM_PACK32,
    )?)?;

    let normals_buffer = ImageView::new_default(AttachmentImage::transient_input_attachment(
        allocator,
        dimensions,
        Format::R16G16B16A16_SFLOAT,
    )?)?;

    let depth_buffer = ImageView::new_default(AttachmentImage::transient(
        allocator,
        dimensions,
        Format::D16_UNORM,
    )?)?;

    let framebuffers = images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;
            let r = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![
                        view,
                        color_buffer.clone(),
                        normals_buffer.clone(),
                        depth_buffer.clone(),
                    ],
                    ..Default::default()
                },
            )?;
            Ok(r)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((framebuffers, color_buffer, normals_buffer))
}
