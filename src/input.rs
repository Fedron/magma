use winit::event::{ElementState, ScanCode, VirtualKeyCode};

#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub virtual_key_code: VirtualKeyCode,
    pub scancode: ScanCode,
}

impl Key {
    pub fn eq_scancode(&self, other: ScanCode) -> bool {
        self.scancode == other
    }

    pub fn eq_virtual_key_code(&self, other: VirtualKeyCode) -> bool {
        self.virtual_key_code == other
    }
}

pub struct KeyboardInput {
    pressed_keys: Vec<Key>,
}

impl KeyboardInput {
    pub fn new() -> KeyboardInput {
        KeyboardInput {
            pressed_keys: Vec::new(),
        }
    }

    pub fn contains_scancode(&self, scancode: ScanCode) -> bool {
        for &key in self.pressed_keys.iter() {
            if key.eq_scancode(scancode) {
                return true;
            }
        }

        false
    }

    pub fn contains_virtual_key_code(&self, key_code: VirtualKeyCode) -> bool {
        for &key in self.pressed_keys.iter() {
            if key.eq_virtual_key_code(key_code) {
                return true;
            }
        }

        false
    }

    fn position(&self, scancode: ScanCode) -> Option<usize> {
        for (index, &key) in self.pressed_keys.iter().enumerate() {
            if key.eq_scancode(scancode) {
                return Some(index);
            }
        }

        None
    }

    pub fn register_input(&mut self, input: winit::event::KeyboardInput) {
        if input.state == ElementState::Released {
            if let Some(index) = self.position(input.scancode)
            {
                self.pressed_keys.remove(index);
            }
        } else if input.state == ElementState::Pressed && !self.contains_scancode(input.scancode) {
            self.pressed_keys.push(Key {
                virtual_key_code: input.virtual_keycode.unwrap(),
                scancode: input.scancode,
            })
        }
    }

    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.contains_virtual_key_code(key)
    }
}
