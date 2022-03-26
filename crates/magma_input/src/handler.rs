use crate::prelude::{KeyCode, KeyState, KeyboardInput};

/// Manages incoming inputs from various sources
#[derive(Debug)]
pub struct InputHandler {
    /// Currently pressed keys
    keyboard_inputs: Vec<KeyboardInput>,
}

impl InputHandler {
    /// Creates a new [`InputHandler`]
    pub fn new() -> InputHandler {
        InputHandler {
            keyboard_inputs: Vec::new(),
        }
    }

    /// Processes a keyboard input, adding or removing the input from the list of currently pressed keys
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

    /// Checks whether a key with the give keycode is currently in the pressed [`KeyState`]
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
