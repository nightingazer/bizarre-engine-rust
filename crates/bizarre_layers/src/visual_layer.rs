use std::sync::Arc;

use anyhow::Result;
use bizarre_core::core_events::WindowResized;
use bizarre_core::input::InputHandler;
use bizarre_core::input::MouseButton;
use bizarre_core::{
    app_events::AppCloseRequestedEvent,
    layer::Layer,
    specs::{self, Builder, Read, ReadStorage, RunNow, System, WorldExt, Write},
    timing::DeltaTime,
};
use bizarre_render::render_components::transform::Transform;
use bizarre_render::render_components::Mesh;
use bizarre_render::vertex::VertexData;
use bizarre_render::{
    render_math::DirectionalLight,
    render_submitter::RenderSubmitter,
    renderer::{create_renderer, Renderer, RendererBackend},
};
use nalgebra_glm::vec3_to_vec4;
use nalgebra_glm::vec4_to_vec3;
use nalgebra_glm::Mat4;
use specs::Join;
use winit::{event_loop::ControlFlow, platform::run_return::EventLoopExtRunReturn};

pub struct VisualLayer {
    event_loop: winit::event_loop::EventLoop<()>,
    _window: Arc<winit::window::Window>,
    renderer: Box<dyn Renderer>,
}

impl VisualLayer {
    pub fn new() -> Result<Self> {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::WindowBuilder::new()
            .with_title("Bizarre Engine")
            .build(&event_loop)
            .unwrap();

        let window = Arc::new(window);

        let renderer = create_renderer(window.clone(), RendererBackend::Vulkan);
        let renderer = match renderer {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to create renderer: {:?}", e);
                return Err(e);
            }
        };

        Ok(Self {
            event_loop,
            _window: window,
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

struct MeshSystem;

impl<'a> System<'a> for MeshSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        ReadStorage<'a, Mesh>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, meshes, transforms) = data;

        let mut transformed_meshes: Vec<Mesh> = Vec::with_capacity(meshes.count());

        for (mesh, transform) in (&meshes, &transforms).join() {
            let model = Mat4::from(transform);

            let vertices = mesh
                .vertices
                .iter()
                .cloned()
                .map(|v| {
                    let mut position = vec3_to_vec4(&v.position);
                    position.w = 1.0;
                    let position = model * position;
                    let position = vec4_to_vec3(&position);
                    let mut normal = vec3_to_vec4(&v.normal);
                    normal.w = 0.0;
                    let normal = model * normal;
                    let normal = vec4_to_vec3(&normal);
                    VertexData {
                        position,
                        normal,
                        color: v.color,
                    }
                })
                .collect::<Vec<_>>();

            let transformed_mesh = Mesh {
                vertices,
                indices: mesh.indices.clone(),
            };

            transformed_meshes.push(transformed_mesh);
        }

        submitter.submit_meshes(&transformed_meshes);
    }
}

impl Layer for VisualLayer {
    fn on_attach(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) -> Result<()> {
        let mut submitter = RenderSubmitter::new();
        submitter.set_clear_color([0.0, 0.0, 0.0, 1.0]);
        submitter.submit_ambient_light(bizarre_render::render_math::AmbientLight {
            color: [0.6, 0.9, 1.0],
            intensity: 0.3,
        });

        world.insert(submitter);

        world.register::<Mesh>();
        world.register::<Transform>();
        world.register::<DirectionalLight>();

        world
            .create_entity()
            .with(DirectionalLight {
                color: [1.0, 0.8, 0.6],
                position: [7.5, 10.0, 10.0],
            })
            .build();

        event_bus.push_event(WindowResized {
            width: self._window.inner_size().width as f32,
            height: self._window.inner_size().height as f32,
        });

        Ok(())
    }

    fn on_update(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut bizarre_core::specs::World,
    ) -> Result<()> {
        let mut lights_sys = LightSystem;
        lights_sys.run_now(world);

        let mut mesh_sys = MeshSystem;
        mesh_sys.run_now(world);

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
                        event_bus.push_event(WindowResized {
                            width: size[0] as f32,
                            height: size[1] as f32,
                        });
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
