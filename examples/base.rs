use std::error::Error;

use magma::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Example App Base")
        .build(&event_loop)
        .expect("Failed to create window");

    let instance = Instance::new()?;
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(Queue::Graphics))
        .build(&instance)?;
    let logical_device = LogicalDevice::new(instance, physical_device, &[])?;

    let _surface = Surface::new(logical_device.instance(), &window)?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}
