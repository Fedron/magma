use std::rc::Rc;

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use magma::prelude::*;

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Example App Base")
        .build(&event_loop)
        .expect("Failed to create window");

    let instance = Instance::new()?;
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(Queue::Graphics))
        .device_extensions(&[DeviceExtension::Swapchain])
        .build(&instance)?;
    let logical_device = Rc::new(LogicalDevice::new(instance, physical_device)?);

    let surface = Surface::new(
        logical_device.instance(),
        logical_device.physical_device(),
        &window,
    )?;
    let swapchain = Swapchain::builder()
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), &surface)?;

    let vertex_shader = ShaderBuilder::new("shaders/simple.vert").build(logical_device.clone())?;
    let fragment_shader =
        ShaderBuilder::new("shaders/simple.frag").build(logical_device.clone())?;

    let pipeline = Pipeline::builder()
        .add_shader(vertex_shader)
        .add_shader(fragment_shader)
        .render_pass(swapchain.render_pass())
        .build(logical_device.clone())?;

    let mut command_pool = CommandPool::new(
        logical_device.clone(),
        logical_device
            .physical_device()
            .queue_family(Queue::Graphics)
            .unwrap(),
    )?;
    command_pool.allocate_buffers(
        swapchain.framebuffers().len() as u32,
        CommandBufferLevel::Primary,
    )?;

    for (index, buffer) in command_pool.buffers_mut().iter_mut().enumerate() {
        buffer.begin()?;
        buffer.set_clear_color((0.01, 0.01, 0.01));

        let extent = swapchain.extent();
        buffer.set_viewport(extent.0 as f32, extent.1 as f32)?;
        buffer.set_scissor(extent.clone())?;

        buffer.begin_render_pass(
            swapchain.render_pass(),
            *swapchain.framebuffers().get(index).unwrap(),
            extent,
        )?;
        buffer.bind_pipeline(&pipeline);
        buffer.draw(3, 1, 0, 0);
        buffer.end_render_pass();

        buffer.end()?;
    }

    event_loop.run_return(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });

    Ok(())
}
