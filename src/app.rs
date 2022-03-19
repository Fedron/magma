use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    camera::Camera,
    entity::Entity,
    renderer::{device::Device, simple_render_system::SimpleRenderSystem, Renderer},
    utils,
};

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit window
    window: Rc<winit::window::Window>,
    /// Handle to logical device
    pub device: Rc<Device>,
    /// Handle to the Vulkan renderer
    renderer: Renderer,
    /// List of all entities in the 'world'
    entities: Vec<Entity>,
    /// Current size of the window in pixels
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a new Renderer
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let window = Rc::new(App::init_window(event_loop));
        let device = Rc::new(Device::new(&window));
        let renderer = Renderer::new(window.clone(), device.clone());
        let window_size = window.inner_size();

        App {
            window,
            device,
            renderer,
            entities: Vec::new(),
            window_size,
        }
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
        let simple_render_system = SimpleRenderSystem::new(
            self.device.clone(),
            self.renderer.get_swapchain_render_pass(),
        );

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
                let aspect = self.renderer.aspect_ratio();
                //let camera = Camera::from_orthographic(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
                let camera = Camera::from_perspective(cgmath::Deg(50.0).into(), aspect, 0.1, 10.0);

                if let Some(command_buffer) = self.renderer.begin_frame() {
                    self.renderer.begin_swapchain_render_pass(command_buffer);
                    simple_render_system.render_entities(
                        command_buffer,
                        &mut self.entities,
                        &camera,
                    );
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
