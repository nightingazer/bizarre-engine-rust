use anyhow::Result;
use bizarre_engine::{
    core::{
        app_builder::{self, AppBuilder},
        layer::Layer,
        schedule::ScheduleType,
        specs::{Builder, WorldExt},
    },
    render::{
        mesh_loader::get_mesh_loader_mut,
        render_components::{MeshComponent, TransformComponent},
        render_math::DirectionalLight,
    },
    specs::{Entities, System, Write, WriteStorage},
};
use nalgebra_glm::vec3;

#[derive(Default)]
pub struct SandboxLayer;

impl Layer for SandboxLayer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        app_builder.add_system(
            ScheduleType::Setup,
            SandboxSetupSystem::default(),
            "sandbox_setup",
            &[],
        );

        Ok(())
    }
}

#[derive(Default)]
pub struct SandboxSetupSystem;

impl<'a> System<'a> for SandboxSetupSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, MeshComponent>,
        WriteStorage<'a, DirectionalLight>,
    );

    fn run(
        &mut self,
        (entities, mut transform_storage, mut mesh_storage, mut light_storage): Self::SystemData,
    ) {
        let monkey_mesh = get_mesh_loader_mut()
            //TODO make a decent error handling for models
            .load_obj("assets/models/monkey.obj".into(), Some(&["monkey".into()]))
            .unwrap()[0];

        let cube_mesh = get_mesh_loader_mut()
            .load_obj("assets/models/cube.obj".into(), Some(&["cube".into()]))
            .unwrap()[0];

        let grid_half_size = 3;
        let step = 3;

        for x in (-grid_half_size..=grid_half_size).step_by(step) {
            for z in (-grid_half_size..=grid_half_size).step_by(step) {
                entities
                    .build_entity()
                    .with(
                        TransformComponent {
                            position: [x as f32, 1.0, z as f32].into(),
                            ..Default::default()
                        },
                        &mut transform_storage,
                    )
                    .with(MeshComponent(cube_mesh), &mut mesh_storage)
                    .build();

                entities
                    .build_entity()
                    .with(
                        TransformComponent {
                            position: [x as f32, 3.0, z as f32].into(),
                            ..Default::default()
                        },
                        &mut transform_storage,
                    )
                    .with(MeshComponent(monkey_mesh), &mut mesh_storage)
                    .build();
            }
        }

        entities
            .build_entity()
            .with(MeshComponent(cube_mesh), &mut mesh_storage)
            .with(
                TransformComponent {
                    scale: vec3(10.0, 0.1, 10.0),
                    position: vec3(0.0, -0.1, 0.0),
                    ..Default::default()
                },
                &mut transform_storage,
            )
            .build();

        entities
            .build_entity()
            .with(
                DirectionalLight {
                    color: [1.0, 0.8, 0.6],
                    position: [7.5, 10.0, 10.0],
                },
                &mut light_storage,
            )
            .build();
    }
}
