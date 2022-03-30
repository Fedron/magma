use crate::{device::Device, prelude::Window};

pub struct Engine {
    window: Window,
    device: Device,
}

impl Engine {
    pub fn new(window: Window) -> Engine {
        let device = Device::new(&window.winit());

        Engine { window, device }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.window.poll_events();
        }
    }
}
