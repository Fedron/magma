//! Demonstrates how to initialize Vulkan and get a triangle drawing to the screen.
//! Also supports resizing the window.

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
    // If you want to see log outputs from magma make sure to initialize a logger of your choice
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    // Magma uses a winit window internally to initialize Vulkan so you will need to create
    // a winit window and event loop
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Triangle")
        .build(&event_loop)?;

    // Create an instance that will allow us to interface with Vulkan
    let instance = Instance::new(&[DebugLayer::KhronosValidation])?;
    // Finds a physical device that is capable of drawing to the screen
    // We require the swapchain extension and a graphics queue if we want to draw to a surface
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
    // Note: When the window gets resized the swapchain will need to be recreated as its extent
    // will no longer match that of the window or surface
    let mut swapchain = Swapchain::builder()
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), &surface)?;

    // We can finally create the graphics pipeline with our shaders
    let vertex_shader = Shader::new("shaders/triangle.vert")?;
    let fragment_shader = Shader::new("shaders/triangle.frag")?;

    let pipeline = Pipeline::<EmptyVertex>::builder()
        .attach_shader(vertex_shader)
        .attach_shader(fragment_shader)
        .render_pass(swapchain.render_pass())
        .build(logical_device.clone())?;

    // The command pool will hold one command buffer for each framebuffer we have in the swapchain.
    // For now we only allocate the command buffers, we will be recording to them each frame
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

    // This is our main loop
    let mut should_close = false;
    let mut is_minimized = false;
    while !should_close {
        // Poll for events
        event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => should_close = true,
                WindowEvent::Resized(size) => {
                    // We need to check for the case that the window was minimized for when we want to recreate the
                    // swapchain as if we try to create framebuffers with a size of (0, 0) then Vulkan will complain
                    // and the app will crash.
                    if size.width == 0 && size.height == 0 {
                        is_minimized = true
                    } else {
                        is_minimized = false
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => *control_flow = ControlFlow::Exit,
            _ => {}
        });

        // If we are minimized, skip the draw loop where would recreate the swapchain since acquire_next_image()
        // would return SwapchainError::Suboptimal. Skipping the draw loop means we won't try to recreate the swapchain
        // until the window has a size of greater than (0, 0)
        if is_minimized {
            continue;
        }

        // Draw loop
        let result = swapchain.acquire_next_image();
        // Usually you will only need to recreate the swapchain if you get back
        // SwapchainError::Suboptimal although in this case we dediced to recreate it no matter
        // what error we get
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

        // With the image index we can record the command buffer belonging to that framebuffer.
        //
        // You don't need to record your framebuffers every frame but as your rendering gets more
        // complex and dynamic rerecording every frame may make it easier.
        //
        // In this case though, recording the command buffers once when we initialized Vulkan would
        // have been enough as we don't change any data from frame to frame.
        let command_buffer = command_pool.buffers_mut().get_mut(image_index).unwrap();
        command_buffer.begin()?; // You need to first begin recording the command buffer
        command_buffer.set_clear_color((0.01, 0.01, 0.01));
        command_buffer.begin_render_pass(
            swapchain.render_pass(),
            *swapchain.framebuffers().get(image_index).unwrap(),
            swapchain.extent(),
        )?; // Make sure to set the render pass first before we set anything else as they get configured for this render pass

        // The swapchain is created with a dynamic viewport and scissor so we need to set them here
        // to the size of the swapchain
        let extent = swapchain.extent();
        command_buffer.set_viewport(extent.0 as f32, extent.1 as f32)?;
        command_buffer.set_scissor(extent.clone())?;

        // We can finally bind our graphics pipeline and issue the draw command
        command_buffer.bind_pipeline(&pipeline);
        command_buffer.draw(3, 1, 0, 0);

        // Finally we need to make sure we end the render pass and then the command buffer
        command_buffer.end_render_pass();
        command_buffer.end()?;

        // With our command buffer recorded, we can submit it to the swapchain which will queue the
        // command buffer and then present the output to the surface
        let result = swapchain.submit_command_buffer(command_buffer, image_index);
        // Same as before, if we get an error from the swapchain we recreate the swapchain
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

    // Once we finish our main loop we need to wait for the logical device to idle.
    // If we don't wait then Vulkan resources will start being destroyed even though the logical
    // device was still using them
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
