use std::{default, path::Path};

use anyhow::Result;
use bizarre_engine::{
    core::{
        layer::Layer,
        specs::{Builder, WorldExt},
        App,
    },
    layers::{camera_layer::CameraLayer, input_layer::InputLayer, visual_layer::VisualLayer},
    log::{app_logger_init, core_logger_init},
    render::{
        render_components::{Mesh, Transform},
        render_math::DirectionalLight,
        vulkan_utils::shader::load_shader,
    },
};
use nalgebra_glm::{quat_angle, quat_angle_axis, quat_axis, quat_euler_angles, vec3, Mat4, Vec3};

struct SandboxLayer;

impl Layer for SandboxLayer {
    fn on_attach(
        &mut self,
        event_bus: &bizarre_engine::events::observer::EventBus,
        world: &mut bizarre_engine::core::specs::World,
    ) -> Result<()> {
        // world
        //     .create_entity()
        //     .with(Transform {
        //         ..Default::default()
        //     })
        //     .with(Mesh::from_obj("assets/models/cube.obj".to_string())?)
        //     .build();

        let grid_half_size = 3;
        let step = 3;

        for x in (-grid_half_size..=grid_half_size).step_by(step) {
            for z in (-grid_half_size..=grid_half_size).step_by(step) {
                world
                    .create_entity()
                    .with(Transform {
                        position: [x as f32, 1.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(Mesh::from_obj("assets/models/cube.obj".to_string())?)
                    .build();

                world
                    .create_entity()
                    .with(Transform {
                        position: [x as f32, 3.0, z as f32].into(),
                        ..Default::default()
                    })
                    .with(Mesh::from_obj("assets/models/monkey.obj".to_string())?)
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

fn print_transform(transform: &Transform, label: &str) {
    println!(
        "{}:\n\t{:?}\n\t{:?}\n\t{:?}",
        label,
        transform.position,
        transform.scale,
        quat_euler_angles(&transform.rotation)
    );
}

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    // let mut app = App::new("Bizarre Engine");
    // let _ = app.add_layer(CameraLayer::default());
    // let _ = app.add_layer(InputLayer::new());

    // let vis_layer = VisualLayer::new().expect("Failed to create visual layer");
    // let _ = app.add_layer(vis_layer);

    // let _ = app.add_layer(SandboxLayer);
    // app.run();

    let deferred_shader = load_shader(Path::new("assets/shaders/deferred.vert")).unwrap();
}
