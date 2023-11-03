use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    specs::{shred::Resource, WorldExt},
};
use bizarre_render::renderer::{create_renderer, Renderer, RendererBackend};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct VisualLayer {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
    renderer: Box<dyn Renderer>,
}

impl VisualLayer {
    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Bizarre Engine")
            .build(&event_loop)
            .unwrap();

        let renderer = create_renderer(&window, RendererBackend::Vulkan).unwrap();
        Self {
            event_loop,
            window,
            renderer,
        }
    }
}

impl Layer for VisualLayer {
    fn on_attach(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) {
    }

    fn on_update(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) {
        self.event_loop
            .run_return(|event, _, control_flow| match event {
                winit::event::Event::MainEventsCleared => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    self.renderer.render(&self.window);
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        event_bus.push_event(AppCloseRequestedEvent {});
                    }
                    _ => (),
                },
                _ => (),
            });
    }

    fn on_detach(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) {
    }
}
