use bizarre_common::resources::DeltaTime;
use bizarre_core::{
    core_events::WindowResized,
    input::{InputHandler, KeyboardKey, KeyboardModifiers, MouseButton},
    layer::Layer,
    schedule::{ScheduleBuilder, ScheduleType},
};

use bizarre_events::{
    event::EventQueue,
    observer::{EventBus, Observer},
};
use bizarre_logger::core_warn;
use bizarre_render::{
    render_components::{free_camera::FreeCameraComponent, ActiveCamera, Camera, CameraProjection},
    render_submitter::RenderSubmitter,
};
use nalgebra_glm::{vec3, Vec2};
use specs::{Builder, Join, Read, ReadStorage, RunNow, System, WorldExt, Write, WriteStorage};

struct CameraSystem {}

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        Read<'a, InputHandler>,
        Read<'a, DeltaTime>,
        Read<'a, EventQueue<WindowResized>>,
        WriteStorage<'a, FreeCameraComponent>,
        ReadStorage<'a, ActiveCamera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, input, delta_time, window_resize_eq, mut cameras, active_camera) = data;

        let delta_time = delta_time.0.as_secs_f32();
        const BASE_CAMERA_SPEED: f32 = 10.0;

        let mut active_cameras = (&mut cameras, &active_camera).join();

        let active_camera = active_cameras.next();

        if active_camera.is_none() {
            core_warn!("ActiveCameraSystem: no active camera found!");
        } else if active_cameras.next().is_some() {
            core_warn!(
                "ActiveCameraSystem: multiple active cameras found! Going with the first one"
            );
        }

        let (camera, _) = active_camera.unwrap();

        let projection_updated = match window_resize_eq.get_events().iter().last() {
            Some(ev) => {
                camera.update_aspect_ratio(ev.width / ev.height);
                true
            }
            None => false,
        };

        let mut view_updated = false;

        if input.is_key_pressed(&KeyboardKey::W, &KeyboardModifiers::empty()) {
            let direction = {
                let mut base = camera.forward();
                base.y = 0.0;
                base.normalize()
            };
            camera.position += direction * BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::S, &KeyboardModifiers::empty()) {
            let direction = {
                let mut base = camera.forward();
                base.y = 0.0;
                base.normalize()
            };
            camera.position -= direction * BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::A, &KeyboardModifiers::empty()) {
            let direction = {
                let mut base = camera.right();
                base.y = 0.0;
                base.normalize()
            };
            camera.position -= direction * BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::D, &KeyboardModifiers::empty()) {
            let direction = {
                let mut base = camera.right();
                base.y = 0.0;
                base.normalize()
            };
            camera.position += direction * BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::Q, &KeyboardModifiers::empty()) {
            camera.position.y -= BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::E, &KeyboardModifiers::empty()) {
            camera.position.y += BASE_CAMERA_SPEED * delta_time;
            view_updated = true;
        }

        if input.is_key_pressed(&KeyboardKey::Z, &KeyboardModifiers::empty()) {
            camera.yaw = 180.0;
            camera.pitch = 0.0;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::X, &KeyboardModifiers::empty()) {
            camera.yaw = 90.0;
            camera.pitch = 0.0;
            view_updated = true;
        }
        if input.is_key_pressed(&KeyboardKey::Y, &KeyboardModifiers::empty()) {
            camera.yaw = 0.0;
            camera.pitch = 90.0;
            view_updated = true;
        }

        if input.is_button_pressed(&MouseButton::Right, &KeyboardModifiers::empty()) {
            let mouse_delta = input.mouse_delta();

            if mouse_delta != Vec2::zeros() {
                camera.yaw += mouse_delta.x * 0.1;
                camera.pitch += -mouse_delta.y * 0.1;
                camera.pitch = camera.pitch.clamp(-89.0, 89.0);
                view_updated = true;
            }
        }

        if view_updated {
            submitter.update_view(camera.get_view_mat());
            submitter.update_camera_forward(camera.forward());
        }
        if projection_updated {
            submitter.update_projection(camera.get_projection_mat());
        }
    }
}

#[derive(Default)]
pub struct CameraLayer;

impl Layer for CameraLayer {
    fn on_attach(
        &mut self,
        app_builder: &mut bizarre_core::app_builder::AppBuilder,
    ) -> anyhow::Result<()> {
        app_builder.world.register::<FreeCameraComponent>();
        app_builder.world.register::<ActiveCamera>();

        let mut camera = FreeCameraComponent::new(CameraProjection::Perspective {
            fovy: 60.0f32.to_radians(),
            aspect: 1.0,
            near: 0.1,
            far: 250.0,
        });

        camera.position = vec3(0.0, 3.0, 15.0);

        camera.yaw = 180.0;

        app_builder
            .world
            .create_entity()
            .with(camera)
            .with(ActiveCamera)
            .build();

        app_builder.add_system(ScheduleType::Frame, CameraSystem {}, "camera_system", &[]);

        Ok(())
    }
}
