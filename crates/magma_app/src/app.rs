use magma_window::prelude::Window;

/// Represents an application that can draw to a window
pub struct App {
    window: Window,
}

impl App {
    /// Creates a new app with a default window
    pub fn new() -> App {
        let window = Window::builder().build();
        App { window }
    }

    /// Runs the app main loop, this includes staring the window's event loop
    pub fn run(self) {
        self.window.run_event_loop();
    }
}
