use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex, Once,
    },
    thread::JoinHandle,
};

use cfg_if::cfg_if;

use crate::logger_impl::{LogMessage, Logger, APP_LOGGER_NAME, CORE_LOGGER_NAME};

pub static mut LOGGER_THREAD_SENDER: Option<Arc<Mutex<Sender<LogMessage>>>> = None;

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
        LOGGER_THREAD_SENDER = Some(Arc::new(Mutex::new(sender)));
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
            .lock()
            .expect("Failed to lock the logger thread sender")
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
