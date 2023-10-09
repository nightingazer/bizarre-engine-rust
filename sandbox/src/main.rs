use bizarre_engine::{
    core::{input::keyboard_codes::KeyboardCode, App},
    log::{app_logger_init, core_logger_init, debug, info},
};

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");

    let mut app = App::default();
    app.run();

    debug!("Keycode: {}", KeyboardCode::A);
}
