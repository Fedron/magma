use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::utils;

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to Window
    window: Window,
    /// Handle to winit EventLoop corresponding to the creating window
    event_loop: EventLoop<()>,
}

impl App {
    /// Creates a new App
    ///
    /// Creates a platform-specific window, loads the Vulkan library and creates a master renderer
    pub fn new() -> App {
        let (window, event_loop) = App::init_window();

        App { window, event_loop }
    }

    /// Initialises a winit window
    ///
    /// Returns the window, and the event loop used by the window
    pub fn init_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(utils::constants::WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                utils::constants::WINDOW_WIDTH,
                utils::constants::WINDOW_HEIGHT,
            ))
            .build(&event_loop)
            .expect("");

        (window, event_loop)
    }

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(self) {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {}
                Event::LoopDestroyed => {}
                _ => {}
            });
    }
}
