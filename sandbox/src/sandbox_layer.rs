use anyhow::Result;
use bizarre_engine::{
    core::{
        app_builder::{self, AppBuilder},
        layer::Layer,
        specs::{Builder, WorldExt},
    },
    render::{
        mesh_loader::get_mesh_loader_mut,
        render_components::{MeshComponent, TransformComponent},
        render_math::DirectionalLight,
    },
};
use nalgebra_glm::vec3;

#[derive(Default)]
pub struct SandboxLayer;

impl Layer for SandboxLayer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        app_builder.world.register::<TransformComponent>();
        app_builder.world.register::<MeshComponent>();
        app_builder.world.register::<DirectionalLight>();

        let monkey_mesh = get_mesh_loader_mut()
            .load_obj("assets/models/monkey.obj".into(), Some(&["monkey".into()]))?[0];

        let cube_mesh = get_mesh_loader_mut()
            .load_obj("assets/models/cube.obj".into(), Some(&["cube".into()]))?[0];

        let grid_half_size = 3;
        let step = 3;

        for x in (-grid_half_size..=grid_half_size).step_by(step) {
            for z in (-grid_half_size..=grid_half_size).step_by(step) {
                app_builder
                    .world
                    .create_entity()
                    .with(TransformComponent {
                        position: [x as f32, 1.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(MeshComponent(cube_mesh))
                    .build();

                app_builder
                    .world
                    .create_entity()
                    .with(TransformComponent {
                        position: [x as f32, 3.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(MeshComponent(monkey_mesh))
                    .build();
            }
        }

        app_builder
            .world
            .create_entity()
            .with(MeshComponent(cube_mesh))
            .with(TransformComponent {
                scale: vec3(10.0, 0.1, 10.0),
                position: vec3(0.0, -0.1, 0.0),
                ..Default::default()
            })
            .build();

        app_builder
            .world
            .create_entity()
            .with(DirectionalLight {
                color: [1.0, 0.8, 0.6],
                position: [7.5, 10.0, 10.0],
            })
            .build();

        Ok(())
    }
}
