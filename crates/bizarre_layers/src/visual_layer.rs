use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    input::{input::InputHandler, mouse_button::MouseButton},
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
        let mut input_handler = world.write_resource::<InputHandler>();

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
                    winit::event::WindowEvent::Resized(size) => {
                        let size = [size.width, size.height];
                        self.renderer.resize(size);
                    }
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        let keycode = input.scancode as u16;
                        let pressed = match input.state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                        input_handler.process_keyboard(keycode, pressed, event_bus);
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        input_handler
                            .process_mouse_move([position.x as f32, position.y as f32], event_bus);
                    }
                    winit::event::WindowEvent::MouseInput { state, button, .. } => {
                        let pressed = match state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                        let button = match button {
                            winit::event::MouseButton::Left => MouseButton::Left,
                            winit::event::MouseButton::Right => MouseButton::Right,
                            winit::event::MouseButton::Middle => MouseButton::Middle,
                            winit::event::MouseButton::Other(id) => {
                                let id: u8 = id.try_into().unwrap_or(u8::MAX);
                                MouseButton::Other(id)
                            }
                        };
                        input_handler.process_mouse_button(button, pressed, event_bus);
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let delta = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => [x, y],
                            winit::event::MouseScrollDelta::PixelDelta(position) => {
                                [position.x as f32, position.y as f32]
                            }
                        };
                        input_handler.process_mouse_scroll(delta);
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
