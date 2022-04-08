//! A very simple example that renders a triangle to the screen using a hardcoded array of positions in the shader.

use std::rc::Rc;

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use magma::prelude::*;

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Triangle")
        .build(&event_loop)?;

    // Create an instance that will allow us to interface with Vulkan
    let instance = Instance::new()?;
    // Finds a physical device that is capable of drawing to the screen
    // We require the swapchain extension and a graphics queue if we want to draw
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(Queue::Graphics))
        .device_extensions(&[DeviceExtension::Swapchain])
        .build(&instance)?;
    // Creates a logical device that will allow us to interface with the physical device
    let logical_device = Rc::new(LogicalDevice::new(instance, physical_device)?);

    // The surface will be where we are going to draw to and requires the swapchain device extension
    // and graphics queue to be enabled on the physical device
    let surface = Surface::new(
        logical_device.instance(),
        logical_device.physical_device(),
        &window,
    )?;
    // Create the swapchain that we will be drawing to
    // Note: If we try to resize the window the swapchain will no longer fit the window and so it would
    // need to be recreated, in this example I have opted to not recreate the swapchain and simple crash
    // the app if the window is resized. See the 'resizing' example to see how to handle resizing.
    let mut swapchain = Swapchain::builder()
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), &surface)?;

    // We can finally create the graphics pipeline with our shaders
    let vertex_shader =
        ShaderBuilder::new("examples/triangle/simple.vert").build(logical_device.clone())?;
    let fragment_shader =
        ShaderBuilder::new("examples/triangle/simple.frag").build(logical_device.clone())?;

    // Note: Since the pipeline depends on the swapchain render pass it would also need to be recreated
    // when the swapchain gets recreated.
    let pipeline = Pipeline::builder()
        .add_shader(vertex_shader)
        .add_shader(fragment_shader)
        .render_pass(swapchain.render_pass())
        .build(logical_device.clone())?;

    // The command pool will hold one command buffer for each framebuffer we have in the swapchain
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

    // We can now record the commands we need to the command buffers
    for (index, buffer) in command_pool.buffers_mut().iter_mut().enumerate() {
        buffer.begin()?;
        buffer.set_clear_color((0.01, 0.01, 0.01));

        // The swapchain is setup with a dynamic viewport and scissor, so event though we aren't using a
        // dynamic viewport and scissor we still need to set them
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

        // Remember to end the swapchain and render pass
        buffer.end_render_pass();
        buffer.end()?;
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {
            let image_index = swapchain
                .acquire_next_image()
                .expect("Failed to get next image index");
            swapchain
                .submit_command_buffer(&command_pool.buffers()[image_index], image_index)
                .expect("Failed to submit command buffer to swapchain");
        }
        Event::LoopDestroyed => {
            // When we want to exit the event loop we need to wait for the GPU to be finished before
            // we start dropping all the variable we have
            logical_device
                .wait_for_idle()
                .expect("Failed to wait for device to idle");
        }
        _ => {}
    });
}
