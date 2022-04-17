//! Demonstrates how you can bind and use Vulkan push constants

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

// This will be our push constant we use in the pipeline, when deriving `UniformBuffer` make to
// also set the stage attribute so that the `Pipeline` will know what stage to expect the push
// constant at
#[derive(UniformBuffer)]
#[ubo(stage = "vertex")]
struct PushConstant {
    pub _offset: [f32; 2],
    pub _color: [f32; 3],
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Push Constant")
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

    let vertex_shader = Shader::new("shaders/push_constant.vert")?;
    let fragment_shader = Shader::new("shaders/push_constant.frag")?;

    // Since we want to use a push constant in our graphics pipeline we need to set the P generic
    // type on the pipeline to a valid struct that derives UniformBuffer
    let pipeline = Pipeline::<SimpleVertex, PushConstant>::builder()
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

    let mut staging_buffer = Buffer::<SimpleVertex, 4>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    )?;
    staging_buffer.map(u64::MAX, 0)?;
    staging_buffer.write(&[
        SimpleVertex {
            position: [-0.5, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        SimpleVertex {
            position: [0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        SimpleVertex {
            position: [0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
        SimpleVertex {
            position: [-0.5, 0.5],
            color: [1.0, 0.0, 1.0],
        },
    ]);

    let mut vertex_buffer = Buffer::<SimpleVertex, 4>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
        MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    vertex_buffer.copy_from(&staging_buffer, &command_pool)?;

    let mut staging_buffer = Buffer::<u32, 6>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    )?;
    staging_buffer.map(u64::MAX, 0)?;
    staging_buffer.write(&[0, 3, 1, 1, 3, 2]);

    let mut index_buffer = Buffer::<u32, 6>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
        MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    index_buffer.copy_from(&staging_buffer, &command_pool)?;

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
        // Once we bind our pipeline, we need to set the push constant data
        pipeline.set_push_constant(&command_buffer, PushConstant {
            _offset: [0.25, -0.25],
            _color: [0.5, 0.5, 0.5],
        });

        command_buffer.bind_vertex_buffer(&vertex_buffer);
        command_buffer.bind_index_buffer(&index_buffer);
        command_buffer.draw_indexed(6, 1, 0, 0, 0);

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
