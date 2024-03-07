use bitflags::bitflags;

use super::{key_codes::KeyboardKey, mouse_button::MouseButton};

bitflags! {
    #[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub enum MouseEvent {
    Pressed {
        button: MouseButton,
        modifiers: KeyboardModifiers,
    },
    Released {
        button: MouseButton,
        modifiers: KeyboardModifiers,
    },
    Moved {
        x: f32,
        y: f32,
    },
    Scrolled {
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    Pressed {
        key: KeyboardKey,
        modifiers: KeyboardModifiers,
    },
    Released {
        key: KeyboardKey,
        modifiers: KeyboardModifiers,
    },
}
