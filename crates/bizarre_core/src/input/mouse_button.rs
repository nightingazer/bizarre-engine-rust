use std::fmt::{Display, Formatter};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left = 0x01,
    Right = 0x02,
    Middle = 0x03,
    Other(u8),
}

impl From<u8> for MouseButton {
    fn from(value: u8) -> Self {
        match value {
            0x01 => MouseButton::Left,
            0x02 => MouseButton::Right,
            0x03 => MouseButton::Middle,
            _ => MouseButton::Other(value),
        }
    }
}

impl Display for MouseButton {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseButton::Left => write!(f, "Left"),
            MouseButton::Right => write!(f, "Right"),
            MouseButton::Middle => write!(f, "Middle"),
            MouseButton::Other(id) => write!(f, "Other({})", id),
        }
    }
}

impl From<MouseButton> for u8 {
    fn from(value: MouseButton) -> Self {
        match value {
            MouseButton::Left => 0x01,
            MouseButton::Right => 0x02,
            MouseButton::Middle => 0x03,
            MouseButton::Other(id) => id,
        }
    }
}
