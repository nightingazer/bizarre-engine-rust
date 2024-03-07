use bizarre_engine::{
    core::App,
    layers::{camera_layer::CameraLayer, input_layer::InputLayer, visual_layer::VisualLayer},
};
use sandbox_layer::SandboxLayer;

mod sandbox_layer;

fn main() {
    let mut app = App::builder()
        .name("Bizarre Engine")
        .with_layer(VisualLayer::default())
        .with_layer(InputLayer::default())
        .with_layer(SandboxLayer::default())
        .with_layer(CameraLayer::default())
        .build()
        .expect("Failed to build the App");

    app.run();
}
