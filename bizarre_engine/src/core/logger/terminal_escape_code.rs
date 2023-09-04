use std::fmt::Display;

pub type TerminalEscapeCode = u8;

pub const RESET: TerminalEscapeCode = 0;
pub const BOLD: TerminalEscapeCode = 1;
pub const DIM: TerminalEscapeCode = 2;
pub const ITALIC: TerminalEscapeCode = 3;
pub const UNDERLINE: TerminalEscapeCode = 4;
pub const BLINK: TerminalEscapeCode = 5;
pub const INVERTED: TerminalEscapeCode = 7;
pub const HIDDEN: TerminalEscapeCode = 8;
pub const STRIKETHROUGH: TerminalEscapeCode = 9;

pub const BLACK: TerminalEscapeCode = 30;
pub const RED: TerminalEscapeCode = 31;
pub const GREEN: TerminalEscapeCode = 32;
pub const YELLOW: TerminalEscapeCode = 33;
pub const BLUE: TerminalEscapeCode = 34;
pub const MAGENTA: TerminalEscapeCode = 35;
pub const CYAN: TerminalEscapeCode = 36;
pub const WHITE: TerminalEscapeCode = 37;

pub const BRIGHT_BLACK: TerminalEscapeCode = 90;
pub const BRIGHT_RED: TerminalEscapeCode = 91;
pub const BRIGHT_GREEN: TerminalEscapeCode = 92;
pub const BRIGHT_YELLOW: TerminalEscapeCode = 93;
pub const BRIGHT_BLUE: TerminalEscapeCode = 94;
pub const BRIGHT_MAGENTA: TerminalEscapeCode = 95;
pub const BRIGHT_CYAN: TerminalEscapeCode = 96;
pub const BRIGHT_WHITE: TerminalEscapeCode = 97;

pub const fn bg_color(color: TerminalEscapeCode) -> TerminalEscapeCode {
    color + 10
}

pub struct TerminalEscapeSequence(pub Vec<TerminalEscapeCode>);

impl Display for TerminalEscapeSequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let codes = self
            .0
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>()
            .join(";");
        write!(f, "\x1b[{}m", codes)
    }
}
