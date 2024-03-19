use std::borrow::Borrow;

use anyhow::Result;
use bizarre_engine::{
    core::{
        app_builder::{self, AppBuilder},
        layer::Layer,
        schedule::ScheduleType,
        specs::{Builder, WorldExt},
    },
    render::{
        material::builtin_materials::default_plain,
        material_loader::{self, MaterialLoader},
        mesh_loader::get_mesh_loader_mut,
        render_components::{MaterialComponent, MeshComponent, TransformComponent},
        render_math::DirectionalLight,
        render_systems::RendererResource,
    },
    specs::{Entities, Read, System, Write, WriteStorage},
};
use nalgebra_glm::vec3;

#[derive(Default)]
pub struct SandboxLayer;

impl Layer for SandboxLayer {
    fn on_attach(&mut self, app_builder: &mut AppBuilder) -> Result<()> {
        app_builder.add_system(
            ScheduleType::Setup,
            SandboxSetupSystem,
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
        Read<'a, RendererResource>,
        Write<'a, MaterialLoader>,
        WriteStorage<'a, TransformComponent>,
        WriteStorage<'a, MeshComponent>,
        WriteStorage<'a, DirectionalLight>,
        WriteStorage<'a, MaterialComponent>,
    );

    fn run(
        &mut self,
        (
            entities,
            renderer,
            mut material_loader,
            mut transform_storage,
            mut mesh_storage,
            mut light_storage,
            mut material_storage,
        ): Self::SystemData,
    ) {
        let monkey_mesh = get_mesh_loader_mut()
            //TODO make a decent error handling for models
            .load_obj("assets/models/monkey.obj".into(), Some(&["monkey".into()]))
            .unwrap()[0];

        let cube_mesh = get_mesh_loader_mut()
            .load_obj("assets/models/cube.obj".into(), Some(&["cube".into()]))
            .unwrap()[0];

        let default_material_instance = {
            let renderer = renderer.lock().unwrap();

            let default_material = default_plain(
                renderer.max_msaa,
                renderer.render_pass.handle,
                &renderer.device,
            )
            .unwrap();
            let default_material = material_loader.add_material(default_material, "default".into());
            let default_material = material_loader.get_material(default_material);
            let instance = renderer.create_material_instance(default_material).unwrap();
            material_loader.add_instance(instance, String::from("default_material_instance"))
        };

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
                    .with(
                        MaterialComponent(default_material_instance),
                        &mut material_storage,
                    )
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
                    .with(
                        MaterialComponent(default_material_instance),
                        &mut material_storage,
                    )
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
            .with(
                MaterialComponent(default_material_instance),
                &mut material_storage,
            )
            .build();

        entities
            .build_entity()
            .with(
                DirectionalLight {
                    direction: [7.5, 10.0, 10.0].into(),
                    color: [0.8, 0.6, 0.3].into(),
                },
                &mut light_storage,
            )
            .build();
        // entities
        //     .build_entity()
        //     .with(
        //         DirectionalLight {
        //             color: [1.0, 0.0, 0.0].into(),
        //             direction: [1.0, 0.0, 0.0].into(),
        //         },
        //         &mut light_storage,
        //     )
        //     .build();
        // entities
        //     .build_entity()
        //     .with(
        //         DirectionalLight {
        //             color: [0.0, 1.0, 0.0].into(),
        //             direction: [0.0, 1.0, 0.0].into(),
        //         },
        //         &mut light_storage,
        //     )
        //     .build();
        // entities
        //     .build_entity()
        //     .with(
        //         DirectionalLight {
        //             color: [0.0, 0.0, 1.0].into(),
        //             direction: [0.0, 0.0, 1.0].into(),
        //         },
        //         &mut light_storage,
        //     )
        //     .build();
    }
}
