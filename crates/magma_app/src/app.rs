use magma_entities::prelude::World;
use magma_window::prelude::Window;

/// Represents an application that can draw to a window
pub struct App {
    window: Window,
    worlds: Vec<World>,
}

impl App {
    /// Creates a new app with a default window
    pub fn new() -> App {
        let window = Window::builder().build();
        App {
            window,
            worlds: Vec::new(),
        }
    }

    pub fn add_world(&mut self, world: World) {
        self.worlds.push(world);
    }

    /// Runs the app main loop, this includes staring the window's event loop
    pub fn run(self) {
        self.window.run_event_loop();
    }
}
