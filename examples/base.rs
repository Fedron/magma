use std::rc::Rc;

use magma::prelude::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const FRAMES_IN_FLIGHT: usize = 2;

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
    let swapchain = Swapchain::new(logical_device.clone());
    let render_pass = RenderPass::new(
        logical_device.clone(),
        swapchain.color_format(),
        swapchain.depth_format(),
    );

    let _framebuffers = {
        let mut framebuffers: Vec<Framebuffer> = Vec::new();
        for image_view in swapchain.image_views() {
            framebuffers.push(Framebuffer::new(
                logical_device.clone(),
                render_pass.vk_handle(),
                image_view,
                swapchain.extent(),
            ));
        }
        framebuffers
    };

    let (_images_available_semaphores, _render_finished_semaphores, _in_flight_fences) = {
        let mut sync_objects = (Vec::new(), Vec::new(), Vec::new());

        for _ in 0..FRAMES_IN_FLIGHT {
            sync_objects.0.push(Semaphore::new(logical_device.clone()));
            sync_objects.1.push(Semaphore::new(logical_device.clone()));
            sync_objects.2.push(Fence::new(logical_device.clone()));
        }

        sync_objects
    };

    let mut command_pool = CommandPool::new(
        logical_device.clone(),
        CommandPoolFlags::TRANSIENT | CommandPoolFlags::RESETTABLE,
        logical_device
            .physical_device()
            .indices()
            .graphics_family
            .unwrap(),
    );
    command_pool.allocate_buffers(FRAMES_IN_FLIGHT as u32, CommandBufferLevel::Primary);

    for buffer in command_pool.buffers_mut().iter_mut() {
        buffer.begin(CommandBufferUsageFlags::SIMULTANEOUS);
        buffer.end();
    }

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {}
        Event::LoopDestroyed => logical_device.wait_for_idle(),
        _ => {}
    });
}
