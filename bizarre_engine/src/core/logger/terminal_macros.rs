pub use crate::core::logger::terminal_escape_code::TerminalEscapeSequence;

#[macro_export]
macro_rules! escape_sequence {
    ($($code:expr),*) => {
        TerminalEscapeSequence{0: vec![$($code),*]}
    };
}

pub use escape_sequence;
