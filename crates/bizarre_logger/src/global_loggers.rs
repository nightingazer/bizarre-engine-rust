use std::{
    sync::{
        mpsc::{channel, sync_channel, Sender},
        Arc, Mutex, Once,
    },
    thread::JoinHandle,
};

use cfg_if::cfg_if;

use crate::{
    log_errors::LogError,
    logger_impl::{LogMessage, Logger, APP_LOGGER_NAME, CORE_LOGGER_NAME},
};

static mut APP_LOGGER_INIT: Once = Once::new();
static mut CORE_LOGGER_INIT: Once = Once::new();

static mut APP_LOGGER: Option<Arc<Mutex<Logger>>> = None;
static mut CORE_LOGGER: Option<Arc<Mutex<Logger>>> = None;

pub static mut LOGGER_THREAD_SENDER: Option<Sender<LogMessage>> = None;

cfg_if! {
    if #[cfg(debug_assertions)] {
        static mut LOGGER_THREAD_INIT_ONCE: Once = Once::new();
        static mut LOGGER_THREAD_JOIN_ONCE: Once = Once::new();
    }
}

static mut LOGGER_THREAD_HANDLE: Option<std::thread::JoinHandle<()>> = None;

pub fn logging_thread_start(loggers: Option<Vec<Logger>>) -> JoinHandle<()> {
    cfg_if! {
        if #[cfg(debug_assertions)] {
            unsafe {
                if LOGGER_THREAD_INIT_ONCE.is_completed() {
                    panic!("logging_thread_start is called more than once");
                }
                LOGGER_THREAD_INIT_ONCE.call_once(|| {});
            }
        }
    };

    let (sender, receiver) = channel::<LogMessage>();

    unsafe {
        LOGGER_THREAD_SENDER = Some(sender);
    }

    let mut logger_map = match loggers {
        Some(loggers) => {
            let mut logger_map = std::collections::HashMap::new();

            for logger in loggers {
                debug_assert!(
                    !logger_map.contains_key(logger.name()),
                    "Logger with name \"{}\" already exists",
                    logger.name()
                );
                logger_map.insert(logger.name(), logger);
            }
            logger_map
        }
        None => std::collections::HashMap::new(),
    };

    if logger_map.get(CORE_LOGGER_NAME).is_none() {
        logger_map.insert(CORE_LOGGER_NAME, Logger::default_core());
    }

    if logger_map.get(APP_LOGGER_NAME).is_none() {
        logger_map.insert(APP_LOGGER_NAME, Logger::default_app());
    }

    let logger_map = logger_map;

    std::thread::spawn(move || loop {
        let msg = receiver.recv().unwrap();

        let logger = match logger_map.get(msg.logger_name) {
            Some(logger) => logger,
            None => {
                eprintln!("Logger with name \"{}\" does not exist", msg.logger_name);
                continue;
            }
        };

        logger.log(msg.level, msg.msg);

        if msg.shutdown && msg.logger_name == "core" {
            break;
        }
    })
}

pub fn logging_thread_join() {
    cfg_if! {
        if #[cfg(debug_assertions)] {
            unsafe {
                if LOGGER_THREAD_JOIN_ONCE.is_completed() {
                    panic!("logging_thread_join is called more than once");
                }
                LOGGER_THREAD_JOIN_ONCE.call_once(|| {});
            }
        }
    };

    unsafe {
        LOGGER_THREAD_SENDER
            .as_ref()
            .unwrap()
            .send(LogMessage {
                logger_name: CORE_LOGGER_NAME,
                level: crate::LogLevel::Info,
                msg: "Shutting down the logger thread".into(),
                shutdown: true,
            })
            .unwrap();
    }

    unsafe {
        LOGGER_THREAD_HANDLE = None;
    }
}

// pub fn app_logger_init(logger: Option<Logger>) -> Result<(), LogError> {
//     unsafe {
//         if APP_LOGGER_INIT.is_completed() {
//             return Err(LogError::AlreadyInitialized("APP_LOGGER".into()));
//         }
//         APP_LOGGER_INIT.call_once(|| match logger {
//             Some(logger) => APP_LOGGER = Some(Arc::new(Mutex::new(logger))),
//             None => {
//                 APP_LOGGER = Some(Arc::new(Mutex::new(Logger {
//                     label: "App",
//                     targets: vec![
//                         LogTarget::Stderr,
//                         LogTarget::Stdout,
//                         LogTarget::File("./app.log"),
//                     ],
//                     ..Default::default()
//                 })))
//             }
//         });
//         Ok(())
//     }
// }

// /// # Safety
// /// - This function must be called only after the `app_logger_init` function.
// pub unsafe fn app_logger() -> Arc<Mutex<Logger>> {
//     if !APP_LOGGER_INIT.is_completed() {
//         panic!("app_logger is called before the app_logger_init");
//     }
//     APP_LOGGER.as_ref().unwrap().clone()
// }

// pub fn core_logger_init(logger: Option<Logger>) -> Result<(), LogError> {
//     unsafe {
//         if CORE_LOGGER_INIT.is_completed() {
//             return Err(LogError::AlreadyInitialized("CORE_LOGGER".into()));
//         }
//         CORE_LOGGER_INIT.call_once(|| match logger {
//             Some(logger) => CORE_LOGGER = Some(Arc::new(Mutex::new(logger))),
//             None => {
//                 CORE_LOGGER = Some(Arc::new(Mutex::new(Logger {
//                     label: "Engine",
//                     targets: vec![
//                         LogTarget::Stderr,
//                         LogTarget::Stdout,
//                         LogTarget::File("./engine.log"),
//                     ],
//                     ..Default::default()
//                 })))
//             }
//         });
//         Ok(())
//     }
// }

// /// # Safety
// /// - This function must be called only after the `core_logger_init` function.
// pub unsafe fn core_logger() -> Arc<Mutex<Logger>> {
//     if !CORE_LOGGER_INIT.is_completed() {
//         panic!("core_logger is called before the core_logger_init");
//     }
//     CORE_LOGGER.as_ref().unwrap().clone()
// }
