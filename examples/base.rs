use std::rc::Rc;

use magma::prelude::*;
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let instance = Instance::new();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create winit window");

    let surface = Surface::new(&instance, &window);
    let physical_device = PhysicalDevice::new(instance.vk_handle(), &surface);
    let logical_device = Rc::new(LogicalDevice::new(instance, surface, physical_device));
    let _swapchain = Swapchain::new(logical_device);
}
