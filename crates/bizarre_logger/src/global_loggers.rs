use std::sync::{Arc, Mutex, Once};

use crate::{
    log_errors::LogError,
    logger_impl::{LogTarget, Logger},
};

static mut APP_LOGGER_INIT: Once = Once::new();
static mut CORE_LOGGER_INIT: Once = Once::new();

static mut APP_LOGGER: Option<Arc<Mutex<Logger>>> = None;
static mut CORE_LOGGER: Option<Arc<Mutex<Logger>>> = None;

pub fn app_logger_init(logger: Option<Logger>) -> Result<(), LogError> {
    unsafe {
        if APP_LOGGER_INIT.is_completed() {
            return Err(LogError::AlreadyInitialized("APP_LOGGER".into()));
        }
        APP_LOGGER_INIT.call_once(|| match logger {
            Some(logger) => APP_LOGGER = Some(Arc::new(Mutex::new(logger))),
            None => {
                APP_LOGGER = Some(Arc::new(Mutex::new(Logger {
                    label: "App",
                    targets: vec![
                        LogTarget::Stderr,
                        LogTarget::Stdout,
                        LogTarget::File("./app.log"),
                    ],
                    ..Default::default()
                })))
            }
        });
        Ok(())
    }
}

pub unsafe fn app_logger() -> Arc<Mutex<Logger>> {
    if !APP_LOGGER_INIT.is_completed() {
        panic!("app_logger is called before the app_logger_init");
    }
    APP_LOGGER.as_ref().unwrap().clone()
}

pub fn core_logger_init(logger: Option<Logger>) -> Result<(), LogError> {
    unsafe {
        if CORE_LOGGER_INIT.is_completed() {
            return Err(LogError::AlreadyInitialized("CORE_LOGGER".into()));
        }
        CORE_LOGGER_INIT.call_once(|| match logger {
            Some(logger) => CORE_LOGGER = Some(Arc::new(Mutex::new(logger))),
            None => {
                CORE_LOGGER = Some(Arc::new(Mutex::new(Logger {
                    label: "Engine",
                    targets: vec![
                        LogTarget::Stderr,
                        LogTarget::Stdout,
                        LogTarget::File("./engine.log"),
                    ],
                    ..Default::default()
                })))
            }
        });
        Ok(())
    }
}

pub unsafe fn core_logger() -> Arc<Mutex<Logger>> {
    if !CORE_LOGGER_INIT.is_completed() {
        panic!("core_logger is called before the core_logger_init");
    }
    CORE_LOGGER.as_ref().unwrap().clone()
}
