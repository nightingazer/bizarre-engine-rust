use std::fmt::Debug;

use crate::vulkan::vulkan_renderer::VulkanRenderer;

pub trait Renderer: Debug {
    fn new(window: &winit::window::Window) -> anyhow::Result<Self>
    where
        Self: Sized;
    fn destroy(&mut self) -> anyhow::Result<()>;

    fn render(&mut self, window: &winit::window::Window) -> anyhow::Result<()>;
    fn on_resize(&mut self, window_size: (u32, u32)) -> anyhow::Result<()>;
}

pub enum RendererBackend {
    Vulkan,
    OpenGL,
    Metal,
    DirectX,
}

pub fn create_renderer(
    window: &winit::window::Window,
    backend: RendererBackend,
) -> anyhow::Result<Box<dyn Renderer>> {
    let renderer = match backend {
        RendererBackend::Vulkan => VulkanRenderer::new(window)?,
        RendererBackend::OpenGL => unimplemented!("OpenGL is not yet supported."),
        RendererBackend::Metal => unimplemented!("Metal is not yet supported."),
        RendererBackend::DirectX => unimplemented!("DirectX is not yet supported."),
    };

    Ok(Box::new(renderer))
}
