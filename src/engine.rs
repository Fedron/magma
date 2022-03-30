use std::rc::Rc;

use crate::{device::Device, prelude::Window, swapchain::Swapchain};

pub struct Engine {
    window: Window,
    device: Rc<Device>,
    swapchain: Swapchain,
}

impl Engine {
    pub fn new(window: Window) -> Engine {
        let device = Rc::new(Device::new(&window.winit()));
        let swapchain = Swapchain::new(device.clone());

        Engine {
            window,
            device,
            swapchain,
        }
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.window.poll_events();
        }
    }
}
