use bizarre_engine::core::logger::logger;
use bizarre_engine::core::logger::{
    log_level::LogLevel, LogTarget::Stdout, LoggerBuilder, APP_LOGGER,
};
use bizarre_engine::{debug, log_to_global};

fn main() {
    logger::init_app_logger(None);
    debug!("Hello, world!");
}
