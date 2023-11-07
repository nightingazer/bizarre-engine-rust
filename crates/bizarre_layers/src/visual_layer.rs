use anyhow::Result;
use bizarre_core::input::InputHandler;
use bizarre_core::input::MouseButton;
use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    specs::{self, Builder, Read, ReadStorage, RunNow, System, WorldExt, Write},
    timing::DeltaTime,
};
use bizarre_render::{
    render_components::CubeMesh,
    render_math::DirectionalLight,
    render_submitter::RenderSubmitter,
    renderer::{create_renderer, Renderer, RendererBackend},
    vertex::CUBE_VERTICES,
};
use winit::{event_loop::ControlFlow, platform::run_return::EventLoopExtRunReturn};

pub struct VisualLayer {
    event_loop: winit::event_loop::EventLoop<()>,
    _window: winit::window::Window,
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
            _window: window,
            renderer,
        }
    }
}

impl Default for VisualLayer {
    fn default() -> Self {
        Self::new()
    }
}

struct CubeSystem;

impl<'a> System<'a> for CubeSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, CubeMesh>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        let (mut submitter, cubes, delta_time) = data;

        let delta_time = delta_time.0;
        let pi = std::f32::consts::PI;
        let rotation_speed = pi * 0.5;

        let mut dir_light = submitter.get_directional_light().clone();

        let light_position: [f32; 3] = {
            let pos = dir_light.position;
            let pos = nalgebra_glm::Vec3::from(pos);
            let pos = nalgebra_glm::rotate_vec3(
                &pos,
                delta_time * rotation_speed,
                &nalgebra_glm::Vec3::y(),
            );
            pos.into()
        };

        dir_light.position = light_position;

        submitter.set_directional_light(dir_light);

        let mut vertices = Vec::from(CUBE_VERTICES);

        for _cube in cubes.join() {
            submitter.submit_vertices(&mut vertices);
        }
    }
}

impl Layer for VisualLayer {
    fn on_attach(
        &mut self,
        _: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) -> Result<()> {
        let mut submitter = RenderSubmitter::new();
        submitter.set_clear_color([0.3, 0.2, 0.5, 1.0]);
        submitter.set_ambient_light(bizarre_render::render_math::AmbientLight {
            color: [0.3, 0.2, 0.5],
            intensity: 0.5,
        });
        submitter.set_directional_light(DirectionalLight {
            color: [1.0, 1.0, 1.0],
            position: [10.0, -10.0, 10.0],
        });

        world.insert(submitter);

        world.register::<CubeMesh>();

        world.create_entity().with(CubeMesh {}).build();

        Ok(())
    }

    fn on_update(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) -> Result<()> {
        let mut cube_sys = CubeSystem;
        cube_sys.run_now(world);
        world.maintain();

        let mut input_handler = world.write_resource::<InputHandler>();

        let mut update_result: Result<()> = Ok(());

        let mut check_result_and_throw = |r: Result<()>, c: &mut ControlFlow| {
            if let Err(e) = r {
                update_result = Err(e);
                *c = ControlFlow::Exit;
            }
        };

        self.event_loop
            .run_return(|event, _, control_flow| match event {
                winit::event::Event::MainEventsCleared => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                    let mut submitter = world.write_resource::<RenderSubmitter>();
                    let render_package = submitter.finalize_submission();
                    let result = self.renderer.render(render_package);
                    check_result_and_throw(result, control_flow);
                }
                winit::event::Event::WindowEvent { event, .. } => match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                        event_bus.push_event(AppCloseRequestedEvent {});
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        let size = [size.width, size.height];
                        let r = self.renderer.resize(size);
                        check_result_and_throw(r, control_flow);
                    }
                    winit::event::WindowEvent::KeyboardInput { input, .. } => {
                        let keycode = input.scancode as u16;
                        let pressed = match input.state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                        let r = input_handler.process_keyboard(keycode, pressed, event_bus);
                        check_result_and_throw(r, control_flow);
                    }
                    winit::event::WindowEvent::CursorMoved { position, .. } => {
                        let r = input_handler
                            .process_mouse_move([position.x as f32, position.y as f32], event_bus);
                        check_result_and_throw(r, control_flow);
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
                        let r = input_handler.process_mouse_button(button, pressed, event_bus);
                        check_result_and_throw(r, control_flow);
                    }
                    winit::event::WindowEvent::MouseWheel { delta, .. } => {
                        let delta = match delta {
                            winit::event::MouseScrollDelta::LineDelta(x, y) => [x, y],
                            winit::event::MouseScrollDelta::PixelDelta(position) => {
                                [position.x as f32, position.y as f32]
                            }
                        };
                        let r = input_handler.process_mouse_scroll(delta);
                        check_result_and_throw(r, control_flow);
                    }
                    _ => (),
                },
                _ => (),
            });

        update_result
    }
}
