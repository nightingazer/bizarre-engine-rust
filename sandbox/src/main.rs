use std::default;

use anyhow::Result;
use bizarre_engine::{
    core::{
        layer::Layer,
        specs::{Builder, WorldExt},
        App,
    },
    layers::{camera_layer::CameraLayer, input_layer::InputLayer, visual_layer::VisualLayer},
    log::{app_logger_init, core_logger_init},
    render::{mesh::Mesh, render_components::Transform},
};

struct CubesLayer;

impl Layer for CubesLayer {
    fn on_attach(
        &mut self,
        event_bus: &bizarre_engine::events::observer::EventBus,
        world: &mut bizarre_engine::core::specs::World,
    ) -> Result<()> {
        world
            .create_entity()
            .with(Transform {
                position: [2.5, 0.0, 0.0].into(),
                ..Default::default()
            })
            .with(Mesh::from_obj("assets/models/cube.obj".to_string())?)
            .build();

        world
            .create_entity()
            .with(Transform {
                position: [-2.5, 0.0, 0.0].into(),
                ..Default::default()
            })
            .with(Mesh::from_obj("assets/models/cube.obj".to_string())?)
            .build();

        world
            .create_entity()
            .with(Transform {
                position: [0.0, -2.5, 0.0].into(),
                ..Default::default()
            })
            .with(Mesh::from_obj("assets/models/monkey.obj".to_string())?)
            .build();

        Ok(())
    }
}

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    let mut app = App::new("Bizarre Engine");
    let _ = app.add_layer(CameraLayer);
    let _ = app.add_layer(InputLayer::new());
    let _ = app.add_layer(VisualLayer::new());

    let _ = app.add_layer(CubesLayer);
    app.run();
}
