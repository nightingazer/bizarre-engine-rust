use anyhow::Result;
use bizarre_engine::{
    core::{
        layer::Layer,
        specs::{Builder, WorldExt},
    },
    render::{
        mesh_loader::get_mesh_loader_mut,
        render_components::{MeshComponent, TransformComponent},
        render_math::DirectionalLight,
    },
};

pub struct SandboxLayer;

impl Layer for SandboxLayer {
    fn on_attach(
        &mut self,
        _event_bus: &bizarre_engine::events::observer::EventBus,
        world: &mut bizarre_engine::core::specs::World,
        _schedule_builder: &mut bizarre_engine::core::schedule::ScheduleBuilder,
    ) -> Result<()> {
        let smooth_monkey_mesh = get_mesh_loader_mut().load_obj(
            "assets/models/smooth_monkey.obj".into(),
            Some(&["smooth_monkey".into()]),
        )?[0];

        let cube_mesh = get_mesh_loader_mut()
            .load_obj("assets/models/cube.obj".into(), Some(&["cube".into()]))?[0];

        let grid_half_size = 3;
        let step = 3;

        for x in (-grid_half_size..=grid_half_size).step_by(step) {
            for z in (-grid_half_size..=grid_half_size).step_by(step) {
                world
                    .create_entity()
                    .with(TransformComponent {
                        position: [x as f32, 1.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(MeshComponent(cube_mesh))
                    .build();

                world
                    .create_entity()
                    .with(TransformComponent {
                        position: [x as f32, 3.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(MeshComponent(smooth_monkey_mesh))
                    .build();
            }
        }

        world
            .create_entity()
            .with(DirectionalLight {
                color: [1.0, 0.8, 0.6],
                position: [7.5, 10.0, 10.0],
            })
            .build();

        world
            .create_entity()
            .with(DirectionalLight {
                color: [0.3, 0.05, 0.35],
                position: [-2.5, 0.1, -5.0],
            })
            .build();

        Ok(())
    }
}
