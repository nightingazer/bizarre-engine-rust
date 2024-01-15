use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use anyhow::Result;
use bizarre_core::core_events::WindowResized;
use bizarre_core::input::InputHandler;
use bizarre_core::input::MouseButton;
use bizarre_core::schedule::ScheduleBuilder;
use bizarre_core::specs;
use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    specs::{ReadStorage, System, WorldExt, Write},
};
use bizarre_events::observer;
use bizarre_events::observer::EventBus;
use bizarre_events::observer::Observer;
use bizarre_render::render_components::transform::TransformComponent;
use bizarre_render::render_components::MeshComponent;
use bizarre_render::render_systems::DrawMeshSystem;
use bizarre_render::render_systems::MeshManagementSystem;
use bizarre_render::Renderer;
use bizarre_render::{render_math::DirectionalLight, render_submitter::RenderSubmitter};
use specs::Join;
use winit::dpi::LogicalPosition;
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::platform::pump_events::PumpStatus;
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::CursorGrabMode;

pub struct VisualLayer {
    event_loop: winit::event_loop::EventLoop<()>,
    window: Arc<winit::window::Window>,
    renderer: Renderer,
}

impl VisualLayer {
    pub fn new() -> Result<Self> {
        std::env::set_var("WINIT_UNIX_BACKEND", "x11");
        let event_loop = winit::event_loop::EventLoop::new()?;
        let window: winit::window::Window = winit::window::WindowBuilder::new()
            .with_title("Bizarre Engine")
            .build(&event_loop)
            .unwrap();

        let window = Arc::new(window);

        let renderer = Renderer::new(window.clone());
        let renderer = match renderer {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to create renderer: {:?}", e);
                return Err(e);
            }
        };

        Ok(Self {
            event_loop,
            window,
            renderer,
        })
    }
}

struct LightSystem;

impl<'a> System<'a> for LightSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, DirectionalLight>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, lights) = data;

        for light in lights.join() {
            submitter.submit_directional_light(light.clone());
        }
    }
}

impl Observer for VisualLayer {
    fn initialize(event_bus: &EventBus, system: observer::SyncObserver<Self>) {
        event_bus.subscribe(system, Self::handle_window_resize);
    }
}

impl VisualLayer {
    fn handle_window_resize(&mut self, event: &WindowResized) {
        self.renderer
            .resize([event.width as u32, event.height as u32])
            .expect("Failed to resize renderer");
    }

    fn handle_event<E>(
        event: winit::event::Event<E>,
        _elwt: &winit::event_loop::EventLoopWindowTarget<E>,
        input_handler: &mut InputHandler,
        event_bus: &EventBus,
        window: &winit::window::Window,
        loop_result: &mut anyhow::Result<()>,
    ) where
        E: 'static,
    {
        use winit::event as w_event;

        if let w_event::Event::WindowEvent { event, .. } = event {
            match event {
                w_event::WindowEvent::CloseRequested => {
                    event_bus.push_event(AppCloseRequestedEvent);
                }
                w_event::WindowEvent::Resized(size) => {
                    let size = [size.width, size.height];
                    event_bus.push_event(WindowResized {
                        width: size[0] as f32,
                        height: size[1] as f32,
                    });
                }
                w_event::WindowEvent::KeyboardInput { event: input, .. } => {
                    let keycode = match input.physical_key {
                        winit::keyboard::PhysicalKey::Code(code) => {
                            code.to_scancode().unwrap() as u16
                        }
                        winit::keyboard::PhysicalKey::Unidentified(code) => match code {
                            winit::keyboard::NativeKeyCode::Xkb(code) => code as u16,
                            _ => u16::MAX,
                        },
                    };
                    let pressed = match input.state {
                        w_event::ElementState::Pressed => true,
                        w_event::ElementState::Released => false,
                    };
                    *loop_result = input_handler.process_keyboard(keycode, pressed, event_bus);
                }
                w_event::WindowEvent::CursorMoved { position, .. } => {
                    *loop_result = input_handler.process_mouse_move(
                        [position.x as f32, position.y as f32].into(),
                        event_bus,
                    );
                }
                w_event::WindowEvent::MouseInput { state, button, .. } => {
                    let pressed = match state {
                        w_event::ElementState::Pressed => true,
                        w_event::ElementState::Released => false,
                    };
                    let button = match button {
                        w_event::MouseButton::Left => MouseButton::Left,
                        w_event::MouseButton::Right => MouseButton::Right,
                        w_event::MouseButton::Middle => MouseButton::Middle,
                        w_event::MouseButton::Other(id) => {
                            let id: u8 = id.try_into().unwrap_or(u8::MAX);
                            MouseButton::Other(id)
                        }
                        _ => MouseButton::Other(u8::MAX),
                    };
                    *loop_result = input_handler.process_mouse_button(button, pressed, event_bus);
                }
                w_event::WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        w_event::MouseScrollDelta::LineDelta(x, y) => [x, y],
                        w_event::MouseScrollDelta::PixelDelta(position) => {
                            [position.x as f32, position.y as f32]
                        }
                    };
                    *loop_result = input_handler.process_mouse_scroll(delta);
                }
                _ => (),
            }
        }
    }
}

impl Layer for VisualLayer {
    fn on_attach(
        &mut self,
        event_bus: &EventBus,
        world: &mut specs::World,
        schedule_builder: &mut ScheduleBuilder,
    ) -> Result<()> {
        event_bus.add_observer(self);

        let mut submitter = RenderSubmitter::new();
        submitter.set_clear_color([0.0, 0.0, 0.0, 1.0]);
        submitter.submit_ambient_light(bizarre_render::render_math::AmbientLight {
            color: [0.6, 0.9, 1.0],
            intensity: 0.3,
        });

        world.insert(submitter);

        world.register::<MeshComponent>();
        world.register::<TransformComponent>();
        world.register::<DirectionalLight>();

        event_bus.push_event(WindowResized {
            width: self.window.inner_size().width as f32,
            height: self.window.inner_size().height as f32,
        });

        let mesh_management_system = MeshManagementSystem {
            reader_id: world.write_storage::<MeshComponent>().register_reader(),
        };

        schedule_builder
            .with_frame_system(mesh_management_system, "mesh_management", &[])
            .with_frame_system(DrawMeshSystem, "draw_meshes", &["mesh_management"]);

        Ok(())
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) -> Result<()> {
        let mut input_handler = world.write_resource::<InputHandler>();

        let timeout = Some(Duration::ZERO);
        let mut result = Ok(());
        let status = self.event_loop.pump_events(timeout, |event, ewlt| {
            Self::handle_event(
                event,
                ewlt,
                &mut input_handler,
                event_bus,
                &self.window,
                &mut result,
            )
        });

        if let Err(e) = result {
            bail!("Failed to handle event: {e}");
        }

        if let PumpStatus::Exit(code) = status {
            bail!("Winit event loop exited with code {code}");
        }

        let mut submitter = world.write_resource::<RenderSubmitter>();
        let render_package = submitter.finalize_submission();
        let result = self.renderer.render(&render_package);

        if let Err(e) = result {
            bail!("Failed to render frame: {e}");
        } else {
            Ok(())
        }
    }
}
