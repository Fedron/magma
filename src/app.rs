use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::utils::constants;

/// Allows you to configure settings for the [App] prior to creating
pub struct AppBuilder {
    window_width: u32,
    window_height: u32,
    window_title: &'static str,
}

impl AppBuilder {
    /// Creates a new [AppBuilder] with default configuration.
    ///
    /// See [constants][crate::utils::constants] module for default values
    pub fn new() -> AppBuilder {
        AppBuilder {
            window_width: constants::WINDOW_WIDTH,
            window_height: constants::WINDOW_HEIGHT,
            window_title: constants::WINDOW_TITLE,
        }
    }

    pub fn window_width(mut self, width: u32) -> AppBuilder {
        self.window_width = width;
        self
    }

    pub fn window_height(mut self, height: u32) -> AppBuilder {
        self.window_height = height;
        self
    }

    pub fn window_title(mut self, title: &'static str) -> AppBuilder {
        self.window_title = title;
        self
    }

    /// Creates a new app from the configuration provided in the builder
    pub fn build(self) -> App {
        let (window, event_loop) =
            App::init_window(self.window_width, self.window_height, self.window_title);

        App { window, event_loop }
    }
}

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit Window
    window: Window,
    /// Handle to winit EventLoop corresponding to the created window
    event_loop: EventLoop<()>,
}

impl App {
    /// Creates a new [AppBuilder] that can be used to configure the values used to create a new [App]
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// Initialises a winit window
    ///
    /// Returns the window, and the event loop used by the window
    pub fn init_window(width: u32, height: u32, title: &'static str) -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
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
