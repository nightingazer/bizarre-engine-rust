use bizarre_logger::core_debug;
use nalgebra_glm::Vec2;
use specs::shrev::EventChannel;

use crate::input::{key_codes::KeyboardKey, KeyboardEvent};

use super::{mouse_button::MouseButton, KeyboardModifiers, MouseEvent};

pub struct InputHandler {
    mouse_previous_position: Vec2,
    mouse_position: Vec2,
    mouse_wheel_delta: Vec2,
    keyboard_modifiers: KeyboardModifiers,
    keyboard_state: [bool; u16::MAX as usize],
    previous_keyboard_state: [bool; u16::MAX as usize],
    mouse_button_state: [bool; u8::MAX as usize],
    previous_mouse_button_state: [bool; u8::MAX as usize],
    pub local_keyboard_eq: Vec<KeyboardEvent>,
    pub local_mouse_eq: Vec<MouseEvent>,
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mouse_position: [0.0, 0.0].into(),
            mouse_previous_position: [0.0, 0.0].into(),
            mouse_wheel_delta: [0.0, 0.0].into(),
            keyboard_modifiers: KeyboardModifiers::NONE,
            keyboard_state: [false; u16::MAX as usize],
            previous_keyboard_state: [false; u16::MAX as usize],
            mouse_button_state: [false; u8::MAX as usize],
            previous_mouse_button_state: [false; u8::MAX as usize],
            local_keyboard_eq: Vec::default(),
            local_mouse_eq: Vec::default(),
        }
    }

    pub fn process_keyboard(&mut self, keycode: u16, pressed: bool) {
        let key = KeyboardKey::from(keycode);
        macro_rules! process_modifiers {
            {$($key:ident => $modifier:ident),+,} => {
                match key {
                    $(
                        KeyboardKey::$key => {
                            if pressed {
                                self.keyboard_modifiers |= KeyboardModifiers::$modifier;
                            } else {
                                self.keyboard_modifiers &= !KeyboardModifiers::$modifier;
                            }
                        }
                    ),+,
                    _ => (),
                }
            }
        }

        process_modifiers! {
            LShift => L_SHIFT,
            LCtrl => L_CTRL,
            LAlt => L_ALT,
            LSuper => L_SUPER,
            RShift => R_SHIFT,
            RCtrl => R_CTRL,
            RAlt => R_ALT,
            RSuper => R_SUPER,
        }

        self.keyboard_state[keycode as usize] = pressed;

        let event = if pressed {
            KeyboardEvent::Pressed {
                key,
                modifiers: self.keyboard_modifiers,
            }
        } else {
            KeyboardEvent::Released {
                key,
                modifiers: self.keyboard_modifiers,
            }
        };

        self.local_keyboard_eq.push(event);
    }

    pub fn process_mouse_move(&mut self, position: Vec2) {
        self.mouse_position = position;

        let event = MouseEvent::Moved {
            x: self.mouse_position[0],
            y: self.mouse_position[1],
        };

        self.local_mouse_eq.push(event);
    }

    pub fn process_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        let index: u8 = button.into();
        self.mouse_button_state[index as usize] = pressed;

        let event = if pressed {
            MouseEvent::Pressed {
                button,
                modifiers: self.keyboard_modifiers,
            }
        } else {
            MouseEvent::Released {
                button,
                modifiers: self.keyboard_modifiers,
            }
        };

        self.local_mouse_eq.push(event);
    }

    pub fn process_mouse_scroll(&mut self, delta: [f32; 2]) {
        self.mouse_wheel_delta[0] += delta[0];
        self.mouse_wheel_delta[1] += delta[1];
    }

    pub fn update(
        &mut self,
        event_queues: &mut (
            &mut EventChannel<MouseEvent>,
            &mut EventChannel<KeyboardEvent>,
        ),
    ) {
        if self.mouse_wheel_delta[0] != 0.0 || self.mouse_wheel_delta[1] != 0.0 {
            let event = MouseEvent::Scrolled {
                x: self.mouse_wheel_delta[0],
                y: self.mouse_wheel_delta[1],
            };

            self.local_mouse_eq.push(event);
        }
        self.mouse_wheel_delta = [0.0, 0.0].into();
        self.mouse_previous_position = self.mouse_position;
        self.previous_keyboard_state = self.keyboard_state;
        self.previous_mouse_button_state = self.mouse_button_state;

        let (mouse, keyboard) = event_queues;

        mouse.drain_vec_write(&mut self.local_mouse_eq);
        keyboard.drain_vec_write(&mut self.local_keyboard_eq);
    }

    pub fn is_key_pressed(&self, key: &KeyboardKey, modifiers: &KeyboardModifiers) -> bool {
        self.keyboard_state[u16::from(*key) as usize]
            && self.keyboard_modifiers.bits() == modifiers.bits()
    }

    pub fn is_button_pressed(&self, button: &MouseButton, modifiers: &KeyboardModifiers) -> bool {
        self.mouse_button_state[u8::from(*button) as usize]
            && self.keyboard_modifiers.bits() == modifiers.bits()
    }

    pub fn mouse_delta(&self) -> Vec2 {
        let mut delta = self.mouse_position;
        delta.x -= self.mouse_previous_position.x;
        delta.y -= self.mouse_previous_position.y;
        delta
    }

    pub fn scroll_delta(&self) -> Vec2 {
        self.mouse_wheel_delta
    }
}
