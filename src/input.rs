use winit::event::{ScanCode, ElementState};

pub struct KeyboardInput {
    pressed_keys: Vec<ScanCode>,
}

impl KeyboardInput {
    pub fn new() -> KeyboardInput {
        KeyboardInput {
            pressed_keys: Vec::new(),
        }
    }

    pub fn register_input(&mut self, input: winit::event::KeyboardInput) {
        if input.state == ElementState::Released {
            if let Some(index) = self
                .pressed_keys
                .iter()
                .position(|&key| key == input.scancode)
            {
                self.pressed_keys.remove(index);
            }
        } else if input.state == ElementState::Pressed
            && !self.pressed_keys.contains(&input.scancode)
        {
            self.pressed_keys.push(input.scancode)
        }
    }

    pub fn is_key_pressed(&self, key: ScanCode) -> bool {
        self.pressed_keys.contains(&key)
    }
}
