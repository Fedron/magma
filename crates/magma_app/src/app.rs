use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use magma_entities::prelude::World;
use magma_input::prelude::InputHandler;
use magma_window::prelude::Window;

/// Represents an application that can draw to a window
pub struct App {
    window: Window,
    worlds: VecDeque<World>,
    pub input_handler: Rc<RefCell<InputHandler>>,
}

impl App {
    /// Creates a new app with a default window
    pub fn new() -> App {
        let window = Window::builder().build();

        App {
            window,
            worlds: VecDeque::new(),
            input_handler: Rc::new(RefCell::new(InputHandler::new())),
        }
    }

    pub fn push_world(&mut self, world: World) {
        self.worlds.push_front(world);
    }

    /// Runs the app main loop, this includes staring the window's event loop
    pub fn run(mut self) {
        self.window
            .run_event_loop(self.input_handler.clone(), move || {
                if self.worlds.len() == 0 {
                    return;
                }

                self.worlds.front_mut().unwrap().update();
                self.worlds.front_mut().unwrap().draw();
            });
    }
}
