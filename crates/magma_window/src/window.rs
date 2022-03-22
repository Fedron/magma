use magma_input::prelude::{InputHandler, KeyCode};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window as WinitWindow, WindowBuilder as WinitWindowBuilder},
};

use crate::events::convert_winit_keyboard_to_magma;

/// Allows you to configure settings for the [_window] prior to creating
pub struct WindowBuilder {
    width: u32,
    height: u32,
    title: &'static str,
}

impl WindowBuilder {
    /// Creates a new [WindowBuilder] with default configuration.
    ///
    /// The default
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            width: 1280,
            height: 720,
            title: "Magma App",
        }
    }

    pub fn width(mut self, width: u32) -> WindowBuilder {
        self.width = width;
        self
    }

    pub fn height(mut self, height: u32) -> WindowBuilder {
        self.height = height;
        self
    }

    pub fn title(mut self, title: &'static str) -> WindowBuilder {
        self.title = title;
        self
    }

    /// Creates a new app from the configuration provided in the builder
    pub fn build(self) -> Window {
        let (window, event_loop) = Window::new_winit(self.width, self.height, self.title);

        Window {
            _window: window,
            event_loop,
            inputs: InputHandler::new(),
        }
    }
}

pub struct Window {
    _window: WinitWindow,
    event_loop: EventLoop<()>,
    inputs: InputHandler,
}

impl Window {
    /// Creates a new [WindowBuilder] with default values
    pub fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }

    pub fn inputs(self) -> InputHandler {
        self.inputs
    }

    /// Initialises a winit _window and event loop
    ///
    /// Returns the _window, and the event loop used by the window
    pub fn new_winit(width: u32, height: u32, title: &'static str) -> (WinitWindow, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let _window = WinitWindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .expect("");

        (_window, event_loop)
    }

    /// Runs the winit event loop
    ///
    /// Blocking operation but returns once the event loop is quit.
    pub fn run_event_loop(mut self) {
        self.event_loop
            .run_return(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        let native_input = convert_winit_keyboard_to_magma(input);
                        self.inputs.process_keyboard_input(native_input);

                        // TODO: Remove
                        if self.inputs.is_key_pressed(KeyCode::Escape) {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    _ => {}
                },
                _ => {}
            });
    }
}
