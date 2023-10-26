use std::sync::mpsc::{channel, Receiver};

use bizarre_logger::{core_debug, core_info, info};
use bizarre_render::renderer::{create_renderer, Renderer, RendererBackend};
use winit::platform::run_return::EventLoopExtRunReturn;

use crate::input::key_codes::KeyboardKey;

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
                winit::event::Event::RedrawEventsCleared if !self.destroying => {
                    self.renderer.render(&self.window).unwrap();
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::Resized(size) => {
                        self.renderer.resize(size.into()).unwrap();
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        self.destroying = true;
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == winit::event::ElementState::Pressed {
                            let keycode = KeyboardKey::from(input.scancode as u16);
                            core_debug!("Keyboard input: {}", keycode);
                        }
                    }
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
