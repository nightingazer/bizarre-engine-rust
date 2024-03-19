use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use bizarre_core::{
    app_builder::AppBuilder,
    app_events::AppCloseRequestedEvent,
    core_events::WindowResized,
    input::{input_handler, InputHandler, MouseButton},
    layer::Layer,
    schedule::ScheduleType,
};
use bizarre_logger::core_debug;
use bizarre_render::{
    material_loader::MaterialLoader,
    render_components::{MeshComponent, WindowComponent},
    render_math::DirectionalLight,
    render_submitter::RenderSubmitter,
    render_systems::{
        MeshDrawRequestSystem, MeshManagementSystem, RendererResource, RendererUpdateSystem,
    },
    scene::RenderScene,
    Renderer,
};
use specs::{
    shrev::EventChannel, Builder, Join, Read, ReadStorage, ReaderId, System, SystemData, WorldExt,
    Write,
};
use winit::{
    dpi::LogicalSize,
    platform::{pump_events::EventLoopExtPumpEvents, scancode::PhysicalKeyExtScancode},
};

#[derive(Default)]
pub struct VisualLayer;

impl Layer for VisualLayer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        let event_loop = winit::event_loop::EventLoop::new()?;

        let window = winit::window::WindowBuilder::new()
            .with_active(true)
            .with_title("Bizarre Engine")
            .with_inner_size(LogicalSize::new(800, 600))
            .build(&event_loop)?;

        let renderer = Renderer::new(&window)?;
        let render_scene = RenderScene::new(renderer.max_frames_in_flight, &renderer.device)?;

        let event_loop = WinitEventLoopResource(Arc::new(Mutex::new(event_loop)));

        app_builder.world.insert(event_loop);
        app_builder.world.insert(RendererResource::new(renderer));
        app_builder.world.insert(MaterialLoader::default());
        app_builder.world.insert(render_scene);

        let mut render_submitter = RenderSubmitter::new();
        render_submitter.submit_ambient_light([0.1, 0.2, 0.4].into());
        app_builder.world.insert(render_submitter);

        app_builder.world.register::<WindowComponent>();
        app_builder
            .world
            .create_entity()
            .with(WindowComponent { handle: window })
            .build();

        app_builder.add_system(
            ScheduleType::Frame,
            WinitEventSystem,
            WinitEventSystem::DEFAULT_NAME,
            &[],
        );

        app_builder.world.register::<MeshComponent>();

        {
            let mesh_reader = app_builder
                .world
                .write_storage::<MeshComponent>()
                .register_reader();
            let mesh_management_system = MeshManagementSystem {
                reader_id: mesh_reader,
            };
            app_builder.add_system(
                ScheduleType::Frame,
                mesh_management_system,
                MeshManagementSystem::DEFAULT_NAME,
                &[],
            );
        }

        app_builder.add_system(
            ScheduleType::Frame,
            MeshDrawRequestSystem::default(),
            MeshDrawRequestSystem::DEFAULT_NAME,
            &[MeshManagementSystem::DEFAULT_NAME],
        );
        app_builder.add_system(
            ScheduleType::Frame,
            LightSystem,
            LightSystem::DEFAULT_NAME,
            &[],
        );
        app_builder.add_system(
            ScheduleType::Frame,
            RendererResizeSystem::default(),
            "renderer_resize",
            &[],
        );
        app_builder.add_barrier(ScheduleType::Frame);
        app_builder.add_system(
            ScheduleType::Frame,
            RendererUpdateSystem,
            RendererUpdateSystem::DEFAULT_NAME,
            &[
                MeshManagementSystem::DEFAULT_NAME,
                LightSystem::DEFAULT_NAME,
            ],
        );

        Ok(())
    }
}

#[derive(Default)]
pub struct RendererResizeSystem {
    reader_id: Option<ReaderId<WindowResized>>,
}

impl<'a> System<'a> for RendererResizeSystem {
    type SystemData = (
        Read<'a, EventChannel<WindowResized>>,
        Write<'a, RendererResource>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (events, mut renderer) = data;
        let events = events.read(self.reader_id.as_mut().unwrap());

        if let Some(resize) = events.last() {
            renderer
                .lock()
                .unwrap()
                .resize([resize.width as u32, resize.height as u32])
        }
    }

    fn setup(&mut self, world: &mut specs::prelude::World) {
        Self::SystemData::setup(world);

        self.reader_id = Some(
            world
                .fetch_mut::<EventChannel<WindowResized>>()
                .register_reader(),
        );
    }
}

pub struct WinitEventLoopResource(pub Arc<Mutex<winit::event_loop::EventLoop<()>>>);

unsafe impl Send for WinitEventLoopResource {}
unsafe impl Sync for WinitEventLoopResource {}

impl Default for WinitEventLoopResource {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(
            winit::event_loop::EventLoop::new().unwrap(),
        )))
    }
}

struct LightSystem;

impl LightSystem {
    pub const DEFAULT_NAME: &'static str = "light_system";
}

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

#[derive(Default)]
pub struct WinitEventSystem;

impl WinitEventSystem {
    pub const DEFAULT_NAME: &'static str = "winit_event_system";
}

impl WinitEventSystem {
    fn handle_event<E: 'static>(
        event: winit::event::Event<E>,
        data: &mut (
            &mut InputHandler,
            &mut EventChannel<AppCloseRequestedEvent>,
            &mut EventChannel<WindowResized>,
        ),
    ) {
        use winit::event as w_event;

        let (input_handler, app_close_channel, window_resize_channel) = data;

        if let w_event::Event::WindowEvent { event, .. } = event {
            match event {
                w_event::WindowEvent::CloseRequested => {
                    app_close_channel.single_write(AppCloseRequestedEvent)
                }
                w_event::WindowEvent::Resized(size) => {
                    let size = [size.width, size.height];
                    window_resize_channel.single_write(WindowResized {
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
                    input_handler.process_keyboard(keycode, pressed);
                }
                w_event::WindowEvent::CursorMoved { position, .. } => {
                    input_handler.process_mouse_move([position.x as f32, position.y as f32].into());
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
                    input_handler.process_mouse_button(button, pressed);
                }
                w_event::WindowEvent::MouseWheel { delta, .. } => {
                    let delta = match delta {
                        w_event::MouseScrollDelta::LineDelta(x, y) => [x, y],
                        w_event::MouseScrollDelta::PixelDelta(position) => {
                            [position.x as f32, position.y as f32]
                        }
                    };
                    input_handler.process_mouse_scroll(delta);
                }
                _ => (),
            }
        }
    }
}

impl<'a> specs::System<'a> for WinitEventSystem {
    type SystemData = (
        Write<'a, WinitEventLoopResource>,
        Write<'a, InputHandler>,
        Write<'a, EventChannel<AppCloseRequestedEvent>>,
        Write<'a, EventChannel<WindowResized>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event_loop, mut input_handler, mut app_close_eq, mut window_resize_eq) = data;

        let mut event_loop = event_loop.0.lock().unwrap();

        event_loop.pump_events(Some(Duration::ZERO), |event, ewlt| {
            Self::handle_event(
                event,
                &mut (&mut input_handler, &mut app_close_eq, &mut window_resize_eq),
            );
        });
    }
}

// impl VisualLayer {
//     fn on_attach(
//         &mut self,
//         event_bus: &EventBus,
//         world: &mut specs::World,
//         schedule_builder: &mut ScheduleBuilder,
//     ) -> Result<()> {
//         let renderer = Renderer::new(&self.window);
//         let renderer = match renderer {
//             Ok(r) => r,
//             Err(e) => {
//                 bail!("Failed to create renderer: {:?}", e);
//             }
//         };
//         self.renderer = Some(renderer);

//         event_bus.add_observer(self);

//         let mut submitter = RenderSubmitter::new();
//         submitter.set_clear_color([0.0, 0.0, 0.0, 1.0]);
//         submitter.submit_ambient_light(bizarre_render::render_math::AmbientLight {
//             color: [0.6, 0.9, 1.0],
//             intensity: 0.3,
//         });

//         world.insert(submitter);

//         world.register::<MeshComponent>();
//         world.register::<TransformComponent>();
//         world.register::<DirectionalLight>();

//         event_bus.push_event(WindowResized {
//             width: self.window.inner_size().width as f32,
//             height: self.window.inner_size().height as f32,
//         });

//         let mesh_management_system = MeshManagementSystem {
//             reader_id: world.write_storage::<MeshComponent>().register_reader(),
//         };

//         schedule_builder
//             .with_frame_system(mesh_management_system, "mesh_management", &[])
//             .with_frame_system(DrawMeshSystem, "draw_meshes", &["mesh_management"]);

//         Ok(())
//     }

//     fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) -> Result<()> {
//         let mut input_handler = world.write_resource::<InputHandler>();

//         let timeout = Some(Duration::ZERO);
//         let mut result = Ok(());
//         let status = self.event_loop.pump_events(timeout, |event, ewlt| {
//             Self::handle_event(
//                 event,
//                 ewlt,
//                 &mut input_handler,
//                 event_bus,
//                 &self.window,
//                 &mut result,
//             )
//         });

//         if let Err(e) = result {
//             bail!("Failed to handle event: {e}");
//         }

//         if let PumpStatus::Exit(code) = status {
//             bail!("Winit event loop exited with code {code}");
//         }

//         let mut submitter = world.write_resource::<RenderSubmitter>();
//         let render_package = submitter.finalize_submission();
//         let result = self
//             .renderer
//             .as_mut()
//             .expect("There is no renderer. The Visual Layer was not initialized properly")
//             .render(&render_package);

//         if let Err(e) = result {
//             bail!("Failed to render frame: {e}");
//         } else {
//             Ok(())
//         }
//     }

//     fn on_detach(&mut self, _event_bus: &EventBus, _world: &mut specs::World) {
//         self.renderer
//             .as_mut()
//             .expect("There is no renderer to destroy")
//             .destroy();
//     }
// }
