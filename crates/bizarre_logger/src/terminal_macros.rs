#[macro_export]
macro_rules! escape_sequence {
    ($($code:expr),*) => {
        TerminalEscapeSequence{0: vec![$($code),*]}
    };
}

#[macro_export]
macro_rules! log_to_global {
    ($logger_getter: tt, $log_level: expr, $msg: expr) => {{
        unsafe {
            $crate::global_loggers::$logger_getter().lock().unwrap().log($log_level, String::from($msg));
        }
    }};
    ($logger_getter: tt, $log_level: expr, $msg:literal, $($args: expr),+) => {{
        let msg = format!($msg, $($args),+);
        $crate::log_to_global!($logger_getter, $log_level, msg);
    }}
}

macro_rules! _gen_log_macro_inner {
    ($logger_getter: tt, $macro_name: tt, $log_level_name: tt) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($$($$args: expr),+) => {
                $crate::log_to_global!($logger_getter, $crate::LogLevel::$log_level_name, $$($$args),+)
            }
        }
    }
}

macro_rules! gen_log_macros {
    ($($logger_getter: tt { $($macro_name: tt => $log_level: tt);+; })+) => {
        $($(_gen_log_macro_inner!($logger_getter, $macro_name, $log_level);)+)+
    };
}

gen_log_macros!(
    core_logger {
        core_debug => Debug;
        core_info => Info;
        core_warn => Warn;
        core_error => Error;
        core_critical => Critical;
    }
    app_logger {
        debug => Debug;
        info => Info;
        warn => Warn;
        error => Error;
        critical => Critical;
    }
);

pub use escape_sequence;
