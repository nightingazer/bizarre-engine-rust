use bizarre_engine::{
    core::App,
    layers::{camera_layer::CameraLayer, input_layer::InputLayer, visual_layer::VisualLayer},
};
use sandbox_layer::SandboxLayer;

mod sandbox_layer;

fn main() {
    let vis_layer = VisualLayer::new().expect("Failed to create visual layer");

    let mut app = App::builder()
        .name("Bizarre Engine")
        .with_layer(CameraLayer::default())
        .with_layer(InputLayer::new())
        .with_layer(vis_layer)
        .with_layer(SandboxLayer)
        .build()
        .expect("Failed to build the App");

    app.run();
}
