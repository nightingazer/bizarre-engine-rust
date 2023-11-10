use bizarre_core::{
    input::{InputHandler, KeyboardModifiers, MouseButton},
    layer::Layer,
};
use bizarre_render::render_submitter::RenderSubmitter;
use nalgebra_glm::{quat_angle, quat_angle_axis, quat_rotate_vec3, rotate, vec3, Quat, Vec2, Vec3};
use specs::{
    Builder, Component, Join, Read, RunNow, System, VecStorage, WorldExt, Write, WriteStorage,
};

pub struct CameraLayer;

pub struct Camera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 5.0,
        }
    }
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}

struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        Read<'a, InputHandler>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, input, mut cameras) = data;

        for camera in (&mut cameras).join() {
            if input.is_button_pressed(&MouseButton::Right, &KeyboardModifiers::NONE) {
                let mouse_delta = Vec2::from(input.mouse_delta()) * 0.01f32;

                camera.yaw -= mouse_delta.x;
                let half_pi = std::f32::consts::PI * 0.5 - 0.01;
                camera.pitch -= mouse_delta.y;
                camera.pitch = camera.pitch.clamp(-half_pi, half_pi);
            }

            let scroll_delta = input.scroll_delta();
            if scroll_delta[1] != 0.0 {
                let zoom_speed = (0.1 * camera.distance).clamp(0.01, 10.0);
                camera.distance -= scroll_delta[1] * zoom_speed;
            }

            let position: Vec3 = vec3(0.0, 0.0, camera.distance);
            let rotation: Quat = quat_angle_axis(camera.yaw, &vec3(0.0, 1.0, 0.0))
                * quat_angle_axis(camera.pitch, &vec3(1.0, 0.0, 0.0));

            let position = quat_rotate_vec3(&rotation, &position);

            submitter.submit_camera_position(position);
        }
    }
}

impl Layer for CameraLayer {
    fn on_attach(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut specs::World,
    ) -> anyhow::Result<()> {
        world.register::<Camera>();

        world.create_entity().with(Camera::default()).build();

        Ok(())
    }

    fn on_update(
        &mut self,
        event_bus: &bizarre_events::observer::EventBus,
        world: &mut specs::World,
    ) -> anyhow::Result<()> {
        let mut camera_sys = CameraSystem;
        camera_sys.run_now(world);
        world.maintain();
        Ok(())
    }
}
