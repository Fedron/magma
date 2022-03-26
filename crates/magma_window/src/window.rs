use std::{cell::RefCell, rc::Rc};

use magma_input::prelude::InputHandler;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window as WinitWindow, WindowBuilder as WinitWindowBuilder},
};

use crate::events::convert_winit_keyboard_to_magma;

/// Allows you to configure settings for a [`Window`] prior to creating it
pub struct WindowBuilder {
    width: u32,
    height: u32,
    title: &'static str,
}

impl WindowBuilder {
    /// Creates a new [WindowBuilder] with a default configuration.
    ///
    /// The default configuration is:
    /// - Width: 1280
    /// - Height: 720
    /// - Title: Magma App
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

    /// Creates a new [`Window`] from the config provided by the [`WindowBuilder`]
    pub fn build(self) -> Window {
        let (window, event_loop) = Window::new_winit(self.width, self.height, self.title);

        Window {
            window: Rc::new(window),
            event_loop,
        }
    }
}

/// Wraps a [`winit::window::Window`] and [`winit::event_loop::EventLoop`]
pub struct Window {
    window: Rc<WinitWindow>,
    event_loop: EventLoop<()>,
}

impl Window {
    /// Creates a new [WindowBuilder] with default values
    pub fn builder() -> WindowBuilder {
        WindowBuilder::new()
    }

    /// Gets the underlying [`winit::window::Window`]
    pub fn winit_window(&self) -> Rc<WinitWindow> {
        self.window.clone()
    }

    /// Creates a new [`winit::window::Window`] and [`winit::event_loop::EventLoop`]
    ///
    /// Returns the window and event loop
    pub fn new_winit(width: u32, height: u32, title: &'static str) -> (WinitWindow, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WinitWindowBuilder::new()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .build(&event_loop)
            .expect("");

        (window, event_loop)
    }

    /// Runs the winit event loop, once all winit events are cleared the `main_loop` is run
    ///
    /// Blocking operation but returns once the event loop is quit.
    pub fn run_event_loop<F>(mut self, input_handler: Rc<RefCell<InputHandler>>, mut main_loop: F)
    where
        F: FnMut(),
    {
        self.event_loop
            .run_return(move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        let native_input = convert_winit_keyboard_to_magma(input);
                        input_handler
                            .borrow_mut()
                            .process_keyboard_input(native_input);
                    }
                    _ => {}
                },
                Event::MainEventsCleared => {
                    main_loop();
                }
                _ => {}
            });
    }
}
