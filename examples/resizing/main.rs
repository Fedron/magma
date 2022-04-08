//! Demonstrates how to handle resizing the swapchain when the window is resized or minimized

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
        .with_title("Resizing")
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
    let mut surface = Surface::new(
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
        ShaderBuilder::new("examples/resizing/simple.vert").build(logical_device.clone())?;
    let fragment_shader =
        ShaderBuilder::new("examples/resizing/simple.frag").build(logical_device.clone())?;

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

    let mut should_close = false;
    while !should_close {
        // Poll for events
        event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => should_close = true,
                _ => {}
            },
            Event::MainEventsCleared => *control_flow = ControlFlow::Exit,
            _ => {}
        });

        // Draw
        let result = swapchain.acquire_next_image();
        if result.is_err() {
            swapchain = recreate_swapchain(
                swapchain,
                logical_device.clone(),
                &mut surface,
                &mut command_pool,
            )?;
            continue;
        }
        let image_index = result.unwrap();

        // To show an alternative method of recording buffers than the method used in the 'triangle' example
        // You can record your buffer every frame if you have large amounts of data changing per frame
        // In this case though since we have a static triangle re-recording every frame in pointless
        let command_buffer = command_pool.buffers_mut().get_mut(image_index).unwrap();
        command_buffer.begin()?;
        command_buffer.set_clear_color((0.01, 0.01, 0.01));
        command_buffer.begin_render_pass(
            swapchain.render_pass(),
            *swapchain.framebuffers().get(image_index).unwrap(),
            swapchain.extent(),
        )?;

        let extent = swapchain.extent();
        command_buffer.set_viewport(extent.0 as f32, extent.1 as f32)?;
        command_buffer.set_scissor(extent.clone())?;

        command_buffer.bind_pipeline(&pipeline);
        command_buffer.draw(3, 1, 0, 0);

        command_buffer.end_render_pass();
        command_buffer.end()?;

        let result = swapchain.submit_command_buffer(command_buffer, image_index);
        if result.is_err() {
            swapchain = recreate_swapchain(
                swapchain,
                logical_device.clone(),
                &mut surface,
                &mut command_pool,
            )?;
            continue;
        }
    }

    logical_device.wait_for_idle()?;

    Ok(())
}

fn recreate_swapchain(
    old_swapchain: Swapchain,
    logical_device: Rc<LogicalDevice>,
    surface: &mut Surface,
    command_pool: &mut CommandPool,
) -> Result<Swapchain> {
    // Before you try to recreate the swapchain you need to wait for the device to stop using it
    logical_device.wait_for_idle()?;
    // We know that the window size has changed and so the surface size will have changed as well
    // But this won't be reflected in the surface's capabilities yet so we need to update those
    surface.update(logical_device.physical_device())?;

    // Create the swapchain just like we do when the app is first run but with the addition of .old_swapchain
    let swapchain = Swapchain::builder()
        .old_swapchain(old_swapchain)
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), surface)?;

    // If for some reason the new swapchain has a different amount of framebuffers we need to ensure
    // our command pool has the same amount
    if swapchain.framebuffers().len() != command_pool.buffers().len() {
        command_pool.free_buffers();
        command_pool.allocate_buffers(
            swapchain.framebuffers().len() as u32,
            CommandBufferLevel::Primary,
        )?;
    }

    Ok(swapchain)
}
