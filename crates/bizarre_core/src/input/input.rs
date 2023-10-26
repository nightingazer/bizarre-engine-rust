use crate::{
    input::input_event::KeyboardModifiers, input::key_codes::KeyboardKey, traits::Updatable,
};

pub struct InputHandler {
    mouse_previous_position: [f32; 2],
    mouse_position: [f32; 2],
    mouse_wheel_delta: [f32; 2],
    keyboard_modifiers: KeyboardModifiers,
    keyboard_state: [bool; u16::MAX as usize],
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mouse_position: [0.0, 0.0],
            mouse_previous_position: [0.0, 0.0],
            mouse_wheel_delta: [0.0, 0.0],
            keyboard_modifiers: KeyboardModifiers::NONE,
            keyboard_state: [false; u16::MAX as usize],
        }
    }

    pub fn process_keyboard(&mut self, keycode: u16, pressed: bool) -> anyhow::Result<()> {
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

        Ok(())
    }
}

impl Updatable for InputHandler {
    fn update(&mut self, delta_time: f32) -> anyhow::Result<()> {
        self.mouse_wheel_delta = [0.0, 0.0];
        self.mouse_previous_position = [self.mouse_position[0], self.mouse_position[1]];
        Ok(())
    }
}
