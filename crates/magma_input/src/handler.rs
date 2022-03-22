use crate::prelude::{KeyCode, KeyState, KeyboardInput};

#[derive(Debug)]
pub struct InputHandler {
    keyboard_inputs: Vec<KeyboardInput>,
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            keyboard_inputs: Vec::new(),
        }
    }

    pub fn process_keyboard_input(&mut self, input: KeyboardInput) {
        for (index, keyboard_input) in self.keyboard_inputs.iter_mut().enumerate() {
            if keyboard_input.scancode == input.scancode {
                if input.state == KeyState::Released {
                    self.keyboard_inputs.remove(index);
                }

                return;
            }
        }

        self.keyboard_inputs.push(input);
    }

    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        for keyboard_input in self.keyboard_inputs.iter() {
            if keyboard_input.state == KeyState::Released {
                continue;
            }

            if let Some(kb_input_keycode) = keyboard_input.keycode {
                if kb_input_keycode == keycode {
                    return true;
                }
            }
        }

        false
    }
}
