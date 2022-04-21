//! Demonstrates how you can use Vulkan descriptors to send data to uniform buffers in your
//! shaders. The end result is the same as the push_constant example but instead descriptors are
//! used

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

// This will be our uniform buffer and it should match the layout of the uniform buffer we define
// in the shader. We also need to set the shader stage the uniform buffer will be used in
#[derive(UniformBuffer)]
#[ubo(stage = "vertex")]
struct Ubo {
    _offset: [f32; 2],
    _color: [f32; 3],
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Descriptors")
        .build(&event_loop)?;

    let instance = Instance::new(&[DebugLayer::KhronosValidation])?;
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(QueueFlags::GRAPHICS))
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

    let vertex_shader = Shader::new("shaders/descriptors.vert")?;
    let fragment_shader = Shader::new("shaders/descriptors.frag")?;

    // We first need to create a descriptor pool from which we will allocate sets
    let descriptor_pool = Rc::new(
        DescriptorPool::builder()
            .add_pool_size(DescriptorType::UniformBuffer, swapchain.framebuffers().len() as u32)
            .max_sets(swapchain.framebuffers().len() as u32)
            .build(logical_device.clone())?
    );

    // We need to define the descriptor set layout, which should match the the descriptor layout
    // you defined in your shaders
    let descriptor_set_layout = Rc::new(DescriptorSetLayout::new(
        logical_device.clone(),
        &[DescriptorSetLayoutBinding {
            binding: 0,
            ty: DescriptorType::UniformBuffer,
            count: 1,
            shader_stage_flags: ShaderStageFlags::VERTEX,
        }],
    )?);

    // We will create a uniform buffer for each framebuffer so that synchronisation is easier.
    // We don't need to write into the buffers at the moment
    let mut ubo_buffers: Vec<Buffer<Ubo, 1>> = Vec::with_capacity(swapchain.framebuffers().len());
    for _ in 0..ubo_buffers.capacity() {
        // When we create the buffer we need to make sure it is flagged as a uniform buffer
        //
        // We also set the host visible and host coherent bits so that we can write to the buffer
        // from the host and have it update on the device without us having to flush the buffer
        // manually
        //
        // Lastly we need the min offset alignment of the buffer to match that of the physical
        // device
        ubo_buffers.push(Buffer::new(
            logical_device.clone(),
            BufferUsageFlags::UNIFORM_BUFFER,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
        )?);
    }

    let mut descriptor_sets = Vec::with_capacity(ubo_buffers.len());
    for i in 0..ubo_buffers.len() {
        // In order for a buffer to be able to write to a desriptor set in needs an appropriate
        // usage flag (UNIFORM_BUFFER in this case) set, we can get the buffer's descriptor buffer
        // info using `.descriptor()` but it will only return descriptor buffer info if the buffer
        // usage flags support it
        descriptor_sets.push(
            DescriptorWriter::new(descriptor_set_layout.clone(), descriptor_pool.clone())
                .write_buffer(0, ubo_buffers[i].descriptor().unwrap())
                .write()?
        );
    }

    let pipeline = Pipeline::<SimpleVertex, EmptyPushConstant>::builder()
        .attach_shader(vertex_shader)
        .attach_shader(fragment_shader)
        .render_pass(swapchain.render_pass())
        .build(logical_device.clone())?;

    let mut command_pool = CommandPool::new(
        logical_device.clone(),
        logical_device
            .physical_device()
            .queue_family(QueueFlags::GRAPHICS)
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

        pipeline.bind(command_buffer);

        // Before we bind the descriptor set we need to write some data to it
        let ubo = ubo_buffers.get_mut(image_index).unwrap();
        ubo.map(u64::MAX, 0)?;
        ubo.write(&[Ubo {
            _offset: [0.25, -0.25],
            _color: [0.5, 0.5, 0.5],
        }]);

        // Lastly, we need to set the descriptor set on the pipeline so that the shader recieves
        // the ubo
        pipeline.bind_descriptor_sets(command_buffer, &[descriptor_sets[image_index]]);
        
        pipeline.draw_indexed(command_buffer, &vertex_buffer, &index_buffer);

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
