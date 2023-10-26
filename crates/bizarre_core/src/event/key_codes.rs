use std::fmt::{Display, Formatter};

#[cfg(target_os = "linux")]
macro_rules! __expand_value {
    ($l:expr, $w:expr) => {
        $l
    };
}

#[cfg(target_os = "windows")]
macro_rules! __expand_value {
    ($l:expr, $w:expr) => {
        $w
    };
}

macro_rules! key_codes {
    {$enum_name:tt : $type:ty {$($name:tt = (L: $l:expr, W: $w:expr)),+,}} => {
        #[repr($type)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $enum_name {
            Unknown($type),
            $($name = __expand_value!($l, $w)),*,
        }

        impl From<$type> for $enum_name {
            fn from(value: $type) -> Self {
                match value {
                    $(__expand_value!($l, $w) => $enum_name::$name),*,
                    _ => $enum_name::Unknown(value),
                }
            }
        }

        impl Display for $enum_name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $enum_name::$name => write!(f, stringify!($name)),
                    )*
                    $enum_name::Unknown(value) => write!(f, "Unknown(0x{:x})", value),
                }
            }
        }
    }
}

key_codes! {
    KeyboardKey : u32 {

        Q = (L: 0x10, W: 0x00),
        W = (L: 0x11, W: 0x00),
        E = (L: 0x12, W: 0x00),
        R = (L: 0x13, W: 0x00),
        T = (L: 0x14, W: 0x00),
        Y = (L: 0x15, W: 0x00),
        U = (L: 0x16, W: 0x00),
        I = (L: 0x17, W: 0x00),
        O = (L: 0x18, W: 0x00),
        P = (L: 0x19, W: 0x00),
        A = (L: 0x1e, W: 0x00),
        S = (L: 0x1f, W: 0x00),
        D = (L: 0x20, W: 0x00),
        F = (L: 0x21, W: 0x00),
        G = (L: 0x22, W: 0x00),
        H = (L: 0x23, W: 0x00),
        J = (L: 0x24, W: 0x00),
        K = (L: 0x25, W: 0x00),
        L = (L: 0x26, W: 0x00),
        Z = (L: 0x2c, W: 0x00),
        X = (L: 0x2d, W: 0x00),
        C = (L: 0x2e, W: 0x00),
        V = (L: 0x2f, W: 0x00),
        B = (L: 0x30, W: 0x00),
        N = (L: 0x31, W: 0x00),
        M = (L: 0x32, W: 0x00),

        Digit1 = (L: 0x02, W: 0x00),
        Digit2 = (L: 0x03, W: 0x00),
        Digit3 = (L: 0x04, W: 0x00),
        Digit4 = (L: 0x05, W: 0x00),
        Digit5 = (L: 0x06, W: 0x00),
        Digit6 = (L: 0x07, W: 0x00),
        Digit7 = (L: 0x08, W: 0x00),
        Digit8 = (L: 0x09, W: 0x00),
        Digit9 = (L: 0x0a, W: 0x00),
        Digit0 = (L: 0x0b, W: 0x00),

        Escape      = (L: 0x01, W: 0x00),
        Backspace   = (L: 0x0e, W: 0x00),
        Enter       = (L: 0x1c, W: 0x00),
        Space       = (L: 0x39, W: 0x00),
        Tab         = (L: 0x0f, W: 0x00),
        CapsLock    = (L: 0x3a, W: 0x00),
        RShift      = (L: 0x36, W: 0x00),
        LShift      = (L: 0x2a, W: 0x00),
        RControl    = (L: 0x64, W: 0x00),
        LControl    = (L: 0x1d, W: 0x00),
        RAlt        = (L: 0x61, W: 0x00),
        LAlt        = (L: 0x38, W: 0x00),
        Super       = (L: 0x7d, W: 0x00),

        Insert      = (L: 0x6e, W: 0x00),
        Delete      = (L: 0x6f, W: 0x00),
        Home        = (L: 0x66, W: 0x00),
        End         = (L: 0x6b, W: 0x00),
        PageUp      = (L: 0x68, W: 0x00),
        PageDown    = (L: 0x6d, W: 0x00),

        ArrowUp     = (L: 0x67, W: 0x00),
        ArrowDown   = (L: 0x6c, W: 0x00),
        ArrowLeft   = (L: 0x69, W: 0x00),
        ArrowRight  = (L: 0x6a, W: 0x00),

        LeftBracket     = (L: 0x1a, W: 0x00),
        RightBracket    = (L: 0x1b, W: 0x00),
        SemiColon       = (L: 0x27, W: 0x00),
        Apostrophe      = (L: 0x28, W: 0x00),
        Comma           = (L: 0x33, W: 0x00),
        Period          = (L: 0x34, W: 0x00),
        Slash           = (L: 0x35, W: 0x00),
        BackSlash       = (L: 0x2b, W: 0x00),
        Minus           = (L: 0x0c, W: 0x00),
        Equal           = (L: 0x0d, W: 0x00),

        GraveAccent = (L: 0x29, W: 0x00),

        F1  = (L: 0x3b, W: 0x00),
        F2  = (L: 0x3c, W: 0x00),
        F3  = (L: 0x3d, W: 0x00),
        F4  = (L: 0x3e, W: 0x00),
        F5  = (L: 0x3f, W: 0x00),
        F6  = (L: 0x40, W: 0x00),
        F7  = (L: 0x41, W: 0x00),
        F8  = (L: 0x42, W: 0x00),
        F9  = (L: 0x43, W: 0x00),
        F10 = (L: 0x44, W: 0x00),
        F11 = (L: 0x57, W: 0x00),
        F12 = (L: 0x58, W: 0x00),
    }
}
