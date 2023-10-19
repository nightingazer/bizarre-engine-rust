use std::sync::mpsc::{channel, Receiver};

use bizarre_logger::{core_critical, core_debug, core_info, info};
use bizarre_render::renderer::{create_renderer, Renderer, RendererBackend};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct AppConfig {
    pub title: Box<str>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Bizarre Engine".into(),
        }
    }
}

#[derive(Debug)]
pub struct App {
    title: Box<str>,

    window: winit::window::Window,
    renderer: Box<dyn Renderer>,
    event_loop: winit::event_loop::EventLoop<()>,

    destroying: bool,

    termination_rx: Receiver<()>,
}

impl Drop for App {
    fn drop(&mut self) {
        info!("Destroying the \"{}\" application", self.title);
        self.destroy().unwrap();
    }
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title(config.title.clone())
            .build(&event_loop)
            .expect("Failed to create window");

        let renderer =
            create_renderer(&window, RendererBackend::Vulkan).expect("Failed to create renderer");

        let (tx, rx) = channel::<()>();

        ctrlc::set_handler(move || tx.send(()).expect("Failed to send termination signal"))
            .expect("Failed to set Ctrl-C handler");

        Self {
            title: config.title,
            event_loop,
            window,
            renderer,
            destroying: false,
            termination_rx: rx,
        }
    }

    pub fn run<'a>(&mut self) {
        core_info!("Running the \"{}\" application", self.title);

        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;

            match event {
                winit::event::Event::MainEventsCleared if !self.destroying => {
                    match self.renderer.render(&self.window) {
                        Ok(_) => (),
                        Err(e) => {
                            core_critical!("Failed to render: {}", e);
                            self.destroying = true;
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                    }
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        self.destroying = true;
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::Resized(_) => match self.renderer.on_resize() {
                        Ok(_) => (),
                        Err(e) => {
                            core_critical!("Failed to resize: {}", e);
                            self.destroying = true;
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                    },
                    _ => (),
                },
                _ => (),
            }

            if self.termination_rx.try_recv().is_ok() {
                self.destroying = true;
                *control_flow = winit::event_loop::ControlFlow::Exit;
            }
        });
    }

    pub fn require_destroy(&mut self) {
        self.destroying = true;
    }

    fn destroy(&mut self) -> anyhow::Result<()> {
        info!("Destroying");

        self.renderer.destroy()?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}
