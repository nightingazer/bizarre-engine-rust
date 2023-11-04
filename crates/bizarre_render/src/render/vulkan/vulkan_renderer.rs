use std::{
    default,
    sync::{Arc, Once},
    time::Instant,
};

use anyhow::{anyhow, Result};
use bizarre_logger::core_debug;
use nalgebra_glm::{half_pi, look_at, perspective, pi, vec3};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::{self, PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, SwapchainImage},
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{
        AllocationCreateInfo, MemoryAllocator, MemoryUsage, StandardMemoryAllocator,
    },
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            rasterization::{CullMode, RasterizationState},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, AcquireError, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
        SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
    VulkanLibrary,
};
use vulkano_win::{create_surface_from_handle_ref, create_surface_from_winit, VkSurfaceBuild};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{render_math::ModelViewProjection, renderer::Renderer};

use super::{
    shaders::{fs, vs},
    vertex::{VertexData, CUBE_VERTICES},
};

pub struct VulkanRenderer {
    instance: Arc<Instance>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    viewport: Viewport,
    queue: Arc<Queue>,

    recreate_swapchain: bool,
    surface_size: [u32; 2],

    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    pipeline: Arc<GraphicsPipeline>,

    previous_frame_end: Option<Box<dyn GpuFuture>>,

    cmd_buffer_allocator: StandardCommandBufferAllocator,
}

impl Renderer for VulkanRenderer {
    fn new(window: &winit::window::Window) -> Result<Self>
    where
        Self: Sized,
    {
        let instance = {
            let library = VulkanLibrary::new()?;
            let extensions = vulkano_win::required_extensions(&library);
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

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )?;

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let allocator: StandardMemoryAllocator =
            StandardMemoryAllocator::new_default(device.clone());

        let framebuffers =
            window_size_dependent_setup(&images, render_pass.clone(), &mut viewport, &allocator)?;

        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

        let vs = vs::load(device.clone())?;
        let fs = fs::load(device.clone())?;

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<VertexData>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .rasterization_state(RasterizationState::new().cull_mode(CullMode::Back))
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())?;

        Ok(Self {
            instance,
            device,
            swapchain,
            viewport,
            queue,

            recreate_swapchain: false,
            surface_size: window.inner_size().into(),

            pipeline,
            framebuffers,
            render_pass,

            previous_frame_end,

            cmd_buffer_allocator,
        })
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
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
            self.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut self.viewport,
                &allocator,
            )?;
            self.recreate_swapchain = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return self.render();
                }
                Err(e) => return Err(anyhow!("Failed to acquire next image: {}", e)),
            };

        if suboptimal {
            self.recreate_swapchain = true;
            return self.render();
        }

        let clear_values = vec![Some([0.075, 0.05, 0.2, 1.0].into()), Some(1.0.into())];

        let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.cmd_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let vertices = CUBE_VERTICES;

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

        let mvp_data = {
            let mut mvp = ModelViewProjection::new();

            let aspect_ratio = self.surface_size[0] as f32 / self.surface_size[1] as f32;
            mvp.projection = perspective(aspect_ratio, half_pi(), 0.01, 100.0);
            mvp.view = look_at(
                &vec3(2.0, -2.0, 2.5),
                &vec3(0.0, 0.0, 0.0),
                &vec3(0.0, 1.0, 0.0),
            );

            crate::render::vulkan::shaders::vs::MVP_Data {
                model: mvp.model.into(),
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

        let ambient_light = crate::render::vulkan::shaders::fs::Ambient_Data {
            ambient_color: [0.3, 0.3, 1.0],
            ambient_intensity: 0.5,
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

        let directional_light = crate::render::vulkan::shaders::fs::Directional_Data {
            color: [1.0, 1.0, 1.0].into(),
            position: [-4.0, -4.0, 5.0].into(),
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

        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(self.device.clone());
        let layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &descriptor_set_allocator,
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, mvp_uniform),
                WriteDescriptorSet::buffer(1, ambient_uniform),
                WriteDescriptorSet::buffer(2, directional_uniform),
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
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set.clone(),
            )
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .draw(vertex_buffer.len() as u32, 1, 0, 0)?
            .end_render_pass()?;

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
) -> Result<Vec<Arc<Framebuffer>>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
    let depth_buffer = ImageView::new_default(AttachmentImage::transient(
        allocator,
        dimensions,
        Format::D16_UNORM,
    )?)?;

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone())?;
            let r = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view, depth_buffer.clone()],
                    ..Default::default()
                },
            )?;
            Ok(r)
        })
        .collect::<Result<Vec<_>>>()
}
