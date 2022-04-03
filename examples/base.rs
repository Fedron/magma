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

    let _surface = Surface::new(&instance, &window);
}
