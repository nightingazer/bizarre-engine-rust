#[macro_export]
macro_rules! escape_sequence {
    ($($code:expr),*) => {
        TerminalEscapeSequence{0: vec![$($code),*]}
    };
}

#[macro_export]
macro_rules! log_to_global {
    ($logger_name: expr, $log_level: expr, $msg: expr) => {{
        unsafe {
            $crate::global_loggers::LOGGER_THREAD_SENDER
                .as_ref()
                .expect("There is no logger thread sender. Make sure that loggin_thread_start() is called before the first attempt to write into a logger")
                .lock()
                .expect("Failed to lock the logger sender")
                .send(
                    $crate::logger_impl::LogMessage {
                        logger_name: $logger_name,
                        level: $log_level,
                        msg: $msg.to_string(),
                        shutdown: false,
                    },
                ).expect("Failed to send log message to global logger");
        }
    }};
    ($logger_name: expr, $log_level: expr, $msg:literal, $($args: expr),+) => {{
        let msg = format!($msg, $($args),+);
        $crate::log_to_global!($logger_name, $log_level, msg);
    }}
}

macro_rules! _gen_log_macro_inner {
    ($logger_name: tt, $macro_name: tt, $log_level_name: tt) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($$($$args: expr),+) => {
                $crate::log_to_global!(stringify!($logger_name), $crate::LogLevel::$log_level_name, $$($$args),+)
            }
        }
    }
}

macro_rules! gen_log_macros {
    ($($logger_name: tt { $($macro_name: tt => $log_level: tt);+; })+) => {
        $($(_gen_log_macro_inner!($logger_name, $macro_name, $log_level);)+)+
    };
}

gen_log_macros!(
    core {
        core_debug => Debug;
        core_info => Info;
        core_warn => Warn;
        core_error => Error;
        core_critical => Critical;
    }
    app {
        debug => Debug;
        info => Info;
        warn => Warn;
        error => Error;
        critical => Critical;
    }
);

pub use escape_sequence;
