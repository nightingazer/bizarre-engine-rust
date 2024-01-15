use bizarre_events::observer::EventBus;
use nalgebra_glm::Vec2;

use crate::{
    input::input_event::KeyboardModifiers,
    input::{input_event::InputEvent, key_codes::KeyboardKey},
};

use super::mouse_button::MouseButton;

pub struct InputHandler {
    mouse_previous_position: Vec2,
    mouse_position: Vec2,
    mouse_wheel_delta: Vec2,
    keyboard_modifiers: KeyboardModifiers,
    keyboard_state: [bool; u16::MAX as usize],
    previous_keyboard_state: [bool; u16::MAX as usize],
    mouse_button_state: [bool; u8::MAX as usize],
    previous_mouse_button_state: [bool; u8::MAX as usize],
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
        }
    }

    pub fn process_keyboard(
        &mut self,
        keycode: u16,
        pressed: bool,
        event_bus: &EventBus,
    ) -> anyhow::Result<()> {
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
            InputEvent::KeyboardPressed {
                key,
                modifiers: self.keyboard_modifiers,
            }
        } else {
            InputEvent::KeyboardReleased {
                key,
                modifiers: self.keyboard_modifiers,
            }
        };

        event_bus.push_event(event);

        Ok(())
    }

    pub fn process_mouse_move(
        &mut self,
        position: Vec2,
        event_bus: &EventBus,
    ) -> anyhow::Result<()> {
        self.mouse_position = position;

        let event = InputEvent::MouseMoved {
            x: self.mouse_position[0],
            y: self.mouse_position[1],
        };

        event_bus.push_event(event);

        Ok(())
    }

    pub fn process_mouse_button(
        &mut self,
        button: MouseButton,
        pressed: bool,
        event_bus: &EventBus,
    ) -> anyhow::Result<()> {
        let index: u8 = button.into();
        self.mouse_button_state[index as usize] = pressed;

        let event = if pressed {
            InputEvent::MousePressed {
                button,
                modifiers: self.keyboard_modifiers,
            }
        } else {
            InputEvent::MouseReleased {
                button,
                modifiers: self.keyboard_modifiers,
            }
        };

        event_bus.push_event(event);

        Ok(())
    }

    pub fn process_mouse_scroll(&mut self, delta: [f32; 2]) -> anyhow::Result<()> {
        self.mouse_wheel_delta[0] += delta[0];
        self.mouse_wheel_delta[1] += delta[1];

        Ok(())
    }

    pub fn update(&mut self, event_bus: &EventBus) -> anyhow::Result<()> {
        if self.mouse_wheel_delta[0] != 0.0 || self.mouse_wheel_delta[1] != 0.0 {
            let event = InputEvent::MouseScrolled {
                x: self.mouse_wheel_delta[0],
                y: self.mouse_wheel_delta[1],
            };

            event_bus.push_event(event);
        }
        self.mouse_wheel_delta = [0.0, 0.0].into();
        self.mouse_previous_position = self.mouse_position;
        self.previous_keyboard_state = self.keyboard_state;
        self.previous_mouse_button_state = self.mouse_button_state;
        Ok(())
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
        delta[0] -= self.mouse_previous_position[0];
        delta[1] -= self.mouse_previous_position[1];
        delta
    }

    pub fn scroll_delta(&self) -> Vec2 {
        self.mouse_wheel_delta
    }
}
