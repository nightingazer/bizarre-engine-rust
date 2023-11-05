use bizarre_engine::{
    core::App,
    layers::{input_layer::InputLayer, visual_layer::VisualLayer},
    log::{app_logger_init, core_logger_init},
};

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    let mut app = App::new("Bizarre Engine");
    app.add_layer(InputLayer::new());
    app.add_layer(VisualLayer::new());
    app.run();
}
