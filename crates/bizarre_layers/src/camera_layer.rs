use bizarre_core::{
    core_events::WindowResized,
    input::{InputHandler, KeyboardKey, KeyboardModifiers, MouseButton},
    layer::Layer,
    schedule::ScheduleBuilder,
    timing::DeltaTime,
};
use bizarre_events::observer::{self, EventBus, Observer};
use bizarre_render::{
    render_components::{Camera, CameraProjection},
    render_submitter::RenderSubmitter,
};
use nalgebra_glm::{quat_angle, quat_angle_axis, quat_axis, vec2, Quat, Vec2, Vec3};
use specs::{Builder, Join, Read, RunNow, System, WorldExt, Write, WriteStorage};

struct CameraSystem {
    updated_aspect_ratio: Option<f32>,
}

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        Write<'a, RenderSubmitter>,
        Read<'a, InputHandler>,
        Read<'a, DeltaTime>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut submitter, input, delta_time, mut cameras) = data;

        let delta_time = delta_time.0.as_secs_f32();
        const BASE_CAMERA_SPEED: f32 = 5.0;

        for camera in (&mut cameras).join() {
            let mut should_submit = false;

            if let Some(aspect) = self.updated_aspect_ratio {
                camera.update_aspect_ratio(aspect);
                should_submit = true;
            }

            if input.is_button_pressed(&MouseButton::Right, &KeyboardModifiers::NONE) {
                let mouse_delta = Vec2::from(input.mouse_delta()) * 0.01f32;
                let mouse_delta = vec2(mouse_delta.y, mouse_delta.x);

                camera.rotate_euler(&mouse_delta);
                let pitch_bound = std::f32::consts::FRAC_PI_2 - 0.001;
                camera.pitch = camera.pitch.clamp(-pitch_bound, pitch_bound);

                should_submit = true;
            }

            if input.is_button_pressed(&MouseButton::Right, &KeyboardModifiers::L_SHIFT) {
                let mut mouse_delta = Vec2::from(input.mouse_delta()) * 0.01;
                mouse_delta.x *= -1.0;
                let position_delta = camera.right() * mouse_delta.x + camera.up() * mouse_delta.y;
                camera.target += position_delta;

                should_submit = true;
            }

            let mut move_direction = Vec3::zeros();

            if input.is_key_pressed(&KeyboardKey::W, &KeyboardModifiers::NONE) {
                move_direction += camera.forward();
            }
            if input.is_key_pressed(&KeyboardKey::S, &KeyboardModifiers::NONE) {
                move_direction -= camera.forward();
            }
            if input.is_key_pressed(&KeyboardKey::A, &KeyboardModifiers::NONE) {
                move_direction -= camera.right();
            }
            if input.is_key_pressed(&KeyboardKey::D, &KeyboardModifiers::NONE) {
                move_direction += camera.right();
            }

            if move_direction != Vec3::zeros() {
                let distance_fraction = camera.distance / 5.0;
                let speed_factor = (distance_fraction * distance_fraction).clamp(0.01, 20.0);
                move_direction = move_direction.normalize();
                move_direction *= BASE_CAMERA_SPEED * speed_factor * delta_time;
                camera.target += move_direction;
                should_submit = true;
            }

            let scroll_delta = input.scroll_delta();
            if scroll_delta[1] != 0.0 {
                let distance_fraction = camera.distance / 5.0;
                let zoom_speed = (distance_fraction * distance_fraction).clamp(0.01, 20.0);
                camera.distance -= scroll_delta[1] * zoom_speed;
                camera.distance = camera.distance.max(0.1);
                should_submit = true;
            }

            if should_submit {
                submitter.update_view(camera.get_view_mat());
                submitter.update_projection(camera.get_projection_mat());
            }
        }
    }
}

#[derive(Default)]
pub struct CameraLayer {
    updated_aspect_ratio: Option<f32>,
}

impl CameraLayer {
    fn handle_resize(&mut self, event: &WindowResized) {
        let aspect_ratio = event.width / event.height;
        self.updated_aspect_ratio = Some(aspect_ratio);
    }
}

impl Layer for CameraLayer {
    fn on_attach(
        &mut self,
        event_bus: &EventBus,
        world: &mut specs::World,
        _schedule_builder: &mut ScheduleBuilder,
    ) -> anyhow::Result<()> {
        world.register::<Camera>();
        let mut camera = Camera::with_projection(CameraProjection::Perspective {
            fovy: 60.0f32.to_radians(),
            aspect: 1.0,
            near: 0.1,
            far: 250.0,
        });

        camera.rotate_euler(&vec2(33.0f32.to_radians(), -45.0f32.to_radians()));
        camera.distance = 7.5;

        world.create_entity().with(camera).build();

        event_bus.add_observer(self);

        Ok(())
    }

    fn on_update(&mut self, event_bus: &EventBus, world: &mut specs::World) -> anyhow::Result<()> {
        let mut camera_sys = CameraSystem {
            updated_aspect_ratio: self.updated_aspect_ratio,
        };
        camera_sys.run_now(world);
        world.maintain();
        self.updated_aspect_ratio = None;
        Ok(())
    }
}

impl Observer for CameraLayer {
    fn initialize(event_bus: &EventBus, system: bizarre_events::observer::SyncObserver<Self>) {
        event_bus.subscribe(system, Self::handle_resize);
    }
}
