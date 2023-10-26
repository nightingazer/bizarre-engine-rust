use bitflags::bitflags;

use super::key_codes::KeyboardKey;

bitflags! {
    pub struct KeyboardModifiers : u8 {
        const NONE = 0b00000000;

        const R_SHIFT   = 0b0000_0001;
        const R_CTRL    = 0b0000_0010;
        const R_ALT     = 0b0000_0100;
        const R_SUPER   = 0b0000_1000;

        const L_SHIFT   = 0b0001_0000;
        const L_CTRL    = 0b0010_0000;
        const L_ALT     = 0b0100_0000;
        const L_SUPER   = 0b1000_0000;

        const SHIFT     = Self::R_SHIFT.bits() | Self::L_SHIFT.bits();
        const CTRL      = Self::R_CTRL.bits()  | Self::L_CTRL.bits();
        const ALT       = Self::R_ALT.bits()   | Self::L_ALT.bits();
        const SUPER     = Self::R_SUPER.bits() | Self::L_SUPER.bits();
    }
}

pub enum InputEvent {
    KeyboardPressed {
        key: KeyboardKey,
        modifiers: KeyboardModifiers,
    },
    KeyboardReleased {
        key: KeyboardKey,
        modifiers: KeyboardModifiers,
    },
    MousePressed {
        button: u8,
    },
    MouseReleased {
        button: u8,
    },
    MouseMoved {
        x: f32,
        y: f32,
    },
    MouseScrolled {
        x: f32,
        y: f32,
    },
}