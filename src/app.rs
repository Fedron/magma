use ash::vk;
use std::{path::Path, rc::Rc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    model::{Model, Vertex},
    utils,
    vulkan::{device::Device, pipeline::Pipeline, swapchain::Swapchain},
};

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit window
    window: winit::window::Window,
    device: Rc<Device>,
    swapchain: Swapchain,
    _pipeline: Pipeline,
    command_buffers: Vec<vk::CommandBuffer>,

    _test_model: Model,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let window = App::init_window(event_loop);
        let device = Rc::new(Device::new(&window));
        let swapchain = Swapchain::new(device.clone());

        let pipeline = Pipeline::new(
            device.clone(),
            Path::new("shaders/simple-shader"),
            swapchain.extent,
            swapchain.render_pass,
        );

        let model = Model::new(
            device.clone(),
            vec![
                Vertex {
                    position: [0.0, -0.5],
                    color: [1.0, 0.0, 0.0],
                },
                Vertex {
                    position: [0.5, 0.5],
                    color: [0.0, 1.0, 0.0],
                },
                Vertex {
                    position: [-0.5, 0.5],
                    color: [0.0, 0.0, 1.0],
                },
            ],
        );

        let command_buffers = App::create_command_buffers(
            &device.device,
            device.command_pool,
            pipeline.graphics_pipeline,
            &swapchain.framebuffers,
            swapchain.render_pass,
            swapchain.extent,
            &model,
        );

        App {
            window,
            device,
            swapchain,
            _pipeline: pipeline,
            command_buffers,
            _test_model: model,
        }
    }

    /// Creates and records new Vulkan command buffers for every framebuffer
    ///
    /// The command buffers only bind a graphics pipeline and draw 3 vertices
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        model: &Model,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.len() as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording to command buffer")
            };

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                })
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline,
                );

                model.bind(command_buffer);
                model.draw(command_buffer);

                device.cmd_end_render_pass(command_buffer);
                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to finish recording command buffer");
            }
        }

        command_buffers
    }

    /// Initialises a winit window, returning the initialised window
    pub fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_title(utils::constants::WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                utils::constants::WINDOW_WIDTH,
                utils::constants::WINDOW_HEIGHT,
            ))
            .build(event_loop)
            .expect("")
    }

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => self.swapchain.draw_frame(&self.command_buffers),
            Event::LoopDestroyed => {
                unsafe {
                    self.device
                        .device
                        .device_wait_idle()
                        .expect("Failed to wait until device idle");
                };
            }
            _ => {}
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_command_pool(self.device.command_pool, None);
        };
    }
}
