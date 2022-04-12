//! Demonstrates how to draw to the screen using a user-defined vertex array
//! See the triangle example for how setting up Vulkan and Magma works

use std::rc::Rc;

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use magma::prelude::*;

#[repr(C)]
#[derive(Vertex)]
struct SimpleVertex {
    #[location = 0]
    position: [f32; 2],
    #[location = 1]
    color: [f32; 3],
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Vertex")
        .build(&event_loop)?;

    let instance = Instance::new(&[DebugLayer::KhronosValidation])?;
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(Queue::Graphics))
        .device_extensions(&[DeviceExtension::Swapchain])
        .build(&instance)?;
    let logical_device = Rc::new(LogicalDevice::new(instance, physical_device)?);

    let mut surface = Surface::new(
        logical_device.instance(),
        logical_device.physical_device(),
        &window,
    )?;
    let mut swapchain = Swapchain::builder()
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), &surface)?;

    // We use `.with_vertex::<V>()` to tell the shader to use our rust struct to represent any in
    // layouts we defined in the shader
    //
    // This will also tell the pipeline to be created with vertex inputs
    let vertex_shader = Shader::new("shaders/vertex.vert")?.with_vertex::<SimpleVertex>();
    let fragment_shader = Shader::new("shaders/vertex.frag")?;

    let pipeline = Pipeline::builder()
        .attach_shader(vertex_shader)
        .attach_shader(fragment_shader)
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

    // We create a staging buffer that will allow us to move the vertex data from the host memory
    // onto the GPU memory which is the fastet memory.
    //
    // We could also just store the vertices in a buffer that is both visible to the host and
    // device but this can start to introduce performance penalties than if you used a device local
    // buffer.
    let mut staging_buffer = Buffer::<SimpleVertex>::new(
        logical_device.clone(),
        3,
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        1
    )?;
    // Before we can write anything to the buffer we need to map it
    staging_buffer.map(u64::MAX, 0)?;
    // After we write, the buffer will automatically be un-mapped
    staging_buffer.write(&[
        SimpleVertex {
            position: [0.0, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        SimpleVertex {
            position: [-0.5, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        SimpleVertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
    ]);

    // We now create a vertex buffer that will only be stored on the GPU and not accesible by us
    // the host, hence why we need the staging buffer to copy from on the device
    let mut vertex_buffer = Buffer::<SimpleVertex>::new(
        logical_device.clone(),
        3,
        BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
        MemoryPropertyFlags::DEVICE_LOCAL,
        1,
    )?;
    // Copy from will copy from a buffer using a command buffer on the devcie
    vertex_buffer.copy_from(&staging_buffer, &command_pool)?;

    let mut should_close = false;
    let mut is_minimized = false;
    while !should_close {
        event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => should_close = true,
                WindowEvent::Resized(size) => {
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

        if is_minimized {
            continue;
        }

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
        // Since our pipeline was created with vertex inputs it now expects a vertex buffer to be
        // bound before we call `draw()`
        command_buffer.bind_vertex_buffer(&vertex_buffer);
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
    logical_device.wait_for_idle()?;
    surface.update(logical_device.physical_device())?;

    let swapchain = Swapchain::builder()
        .old_swapchain(old_swapchain)
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), surface)?;

    if swapchain.framebuffers().len() != command_pool.buffers().len() {
        command_pool.free_buffers();
        command_pool.allocate_buffers(
            swapchain.framebuffers().len() as u32,
            CommandBufferLevel::Primary,
        )?;
    }

    Ok(swapchain)
}
