use anyhow::Result;
use bizarre_core::input::InputHandler;
use bizarre_core::input::MouseButton;
use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    specs::{self, Builder, Read, ReadStorage, RunNow, System, WorldExt, Write},
    timing::DeltaTime,
};
use bizarre_render::render_components::Transform;
use bizarre_render::vertex::VertexData;
use bizarre_render::{
    render_components::CubeMesh,
    render_math::DirectionalLight,
    render_submitter::RenderSubmitter,
    renderer::{create_renderer, Renderer, RendererBackend},
    vertex::CUBE_VERTICES,
};
use nalgebra_glm::radians;
use nalgebra_glm::rotate;
use nalgebra_glm::Mat4;
use nalgebra_glm::TMat4;
use nalgebra_glm::Vec3;
use specs::Join;
use specs::WriteStorage;
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

struct CubeSystem;

impl<'a> System<'a> for CubeSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, CubeMesh>,
        WriteStorage<'a, Transform>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, cubes, mut transforms, delta_time) = data;

        let delta_time = delta_time.0;
        let rotation_speed = 90.0f32;

        for (_cube, transform) in (&cubes, &mut transforms).join() {
            transform.rotation[1] += rotation_speed * delta_time;
            let axis = Vec3::y_axis();
            let angle = transform.rotation[1] * std::f32::consts::PI / 180.0;
            let rotation = rotate(&TMat4::identity(), angle, &axis);

            let vertices = CUBE_VERTICES
                .to_vec()
                .iter()
                .map(|vertex| {
                    let vec_3_pos = Vec3::from(vertex.position);
                    let vec_3_pos = rotation.transform_vector(&vec_3_pos);
                    let position: [f32; 3] = vec_3_pos.into();
                    let vec_3_normal = Vec3::from(vertex.normal);
                    let vec_3_normal = rotation.transform_vector(&vec_3_normal);
                    let normal: [f32; 3] = vec_3_normal.into();
                    VertexData {
                        color: vertex.color,
                        position,
                        normal,
                    }
                })
                .collect::<Vec<_>>();

            submitter.submit_vertices(vertices);
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
        submitter.set_clear_color([0.0, 0.0, 0.0, 1.0]);
        submitter.set_ambient_light(bizarre_render::render_math::AmbientLight {
            color: [0.1, 0.1, 0.1],
            intensity: 0.5,
        });

        world.insert(submitter);

        world.register::<CubeMesh>();
        world.register::<Transform>();
        world.register::<DirectionalLight>();

        world
            .create_entity()
            .with(CubeMesh {})
            .with(Transform::default())
            .build();
        world
            .create_entity()
            .with(DirectionalLight {
                color: [0.4, 0.1, 0.1],
                position: [0.0, -10.0, 5.0],
            })
            .build();
        world
            .create_entity()
            .with(DirectionalLight {
                color: [0.1, 0.4, 0.1],
                position: [-10.0, 0.0, 5.0],
            })
            .build();
        world
            .create_entity()
            .with(DirectionalLight {
                color: [0.1, 0.1, 0.4],
                position: [10.0, 0.0, 5.0],
            })
            .build();

        Ok(())
    }

    fn on_update(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) -> Result<()> {
        let mut cube_sys = CubeSystem;
        cube_sys.run_now(world);
        let mut lights_sys = LightSystem;
        lights_sys.run_now(world);
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
