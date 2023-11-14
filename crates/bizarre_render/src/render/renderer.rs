use std::sync::Arc;

use anyhow::Result;

use crate::{render_package::RenderPackage, vulkan::VulkanRenderer};

pub trait Renderer {
    fn new(window: Arc<winit::window::Window>) -> Result<Self>
    where
        Self: Sized;
    fn destroy(&self) -> Result<()>;

    fn render(&mut self, render_package: RenderPackage) -> Result<()>;

    fn resize(&mut self, size: [u32; 2]) -> Result<()>;
}

pub enum RendererBackend {
    Vulkan,
    OpenGL,
    Metal,
    DirectX,
}

pub fn create_renderer(
    window: Arc<winit::window::Window>,
    backend: RendererBackend,
) -> Result<Box<dyn Renderer>> {
    let renderer = match backend {
        RendererBackend::Vulkan => VulkanRenderer::new(window)?,
        RendererBackend::OpenGL => unimplemented!("OpenGL is not yet supported."),
        RendererBackend::Metal => unimplemented!("Metal is not yet supported."),
        RendererBackend::DirectX => unimplemented!("DirectX is not yet supported."),
    };

    Ok(Box::new(renderer))
}
