use ash::vk;
use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    entity::Entity,
    renderer::{
        device::Device,
        pipeline::{Align16, Pipeline, PushConstants},
        Renderer,
    },
    utils,
};

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit window
    window: Rc<winit::window::Window>,
    /// Handle to logical device
    pub device: Rc<Device>,
    /// Handle to the current graphics pipeline
    pipeline: Pipeline,
    renderer: Renderer,
    /// List of all entities in the 'world'
    entities: Vec<Entity>,
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let window = Rc::new(App::init_window(event_loop));
        let device = Rc::new(Device::new(&window));

        let renderer = Renderer::new(window.clone(), device.clone());
        let pipeline = Pipeline::new(device.clone(), renderer.get_swapchain_render_pass());

        let window_size = window.inner_size();

        App {
            window,
            device,
            pipeline,
            renderer,
            entities: Vec::new(),
            window_size,
        }
    }

    fn render_entities(&mut self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipeline,
            );

            for entity in self.entities.iter_mut() {
                entity.model().bind(command_buffer);
                entity.transform.rotation += 0.1;

                let push = PushConstants {
                    transform: Align16(entity.transform_matrix()),
                    translation: Align16(entity.transform.position),
                };

                self.device.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push.as_bytes(),
                );

                entity.model().draw(command_buffer);
            }
        };
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

    /// Adds a new entity that will be rendered
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    self.window_size = size;
                    self.renderer.recreate_swapchain();
                }
                _ => {}
            },
            Event::MainEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => {
                if let Some(command_buffer) = self.renderer.begin_frame() {
                    self.renderer.begin_swapchain_render_pass(command_buffer);
                    self.render_entities(command_buffer);
                    self.renderer.end_swapchain_render_pass(command_buffer);
                    self.renderer.end_frame();
                }
            }
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
