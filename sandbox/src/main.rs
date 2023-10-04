use bizarre_engine::log::{
    app_logger_init, core_critical, core_debug, core_error, core_info, core_logger_init, core_warn,
    critical, debug, error, info, warn,
};

fn main() {
    core_logger_init(None).expect("Failed to init core logger");
    app_logger_init(None).expect("Failed to init app logger");
    core_debug!("Wow! macros!");
    core_info!("Wow! info macro!");
    core_warn!("Wow! WARNING");
    core_error!("Wow, error macro!");
    core_critical!("WOWOWOWOW, THIS IS CRITICAL!");

    debug!("And for the app side!");
    info!("We are informing, that");
    warn!("All the warnings");
    error!("Which are not error now");
    critical!("Will become CRITICAL SOME DAY");

    core_info!("Hello, world {}", 1);
}
