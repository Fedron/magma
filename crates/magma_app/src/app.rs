use std::{cell::RefCell, collections::HashMap, path::Path, rc::Rc};

use magma_entities::prelude::World;
use magma_input::prelude::InputHandler;
use magma_render::prelude::{Pipeline, PushConstantData, RenderPipeline, Renderer, Vertex};
use magma_window::prelude::Window;

/// Represents an application that can draw to a window
pub struct App {
    window: Window,
    renderer: Renderer,
    worlds: HashMap<Rc<World>, Vec<Box<dyn RenderPipeline>>>,
    active_world: Option<Rc<World>>,
    pub input_handler: Rc<RefCell<InputHandler>>,
}

impl App {
    /// Creates a new app with a default window
    pub fn new() -> App {
        let window = Window::builder().build();
        let renderer = Renderer::new(window.winit_window());

        App {
            window,
            renderer,
            worlds: HashMap::new(),
            active_world: None,
            input_handler: Rc::new(RefCell::new(InputHandler::new())),
        }
    }

    pub fn create_render_pipeline<P: 'static, V: 'static>(
        &mut self,
        vertex_shader: &Path,
        fragment_shader: &Path,
    ) -> Pipeline<P, V>
    where
        P: PushConstantData,
        V: Vertex,
    {
        self.renderer
            .create_pipeline::<P, V>(vertex_shader, fragment_shader)
    }

    pub fn add_world(&mut self, world: Rc<World>) {
        self.worlds.insert(world.clone(), Vec::new());
    }

    pub fn set_active_world(&mut self, world: Rc<World>) {
        self.active_world = Some(world.clone());
    }

    pub fn add_render_pipeline(&mut self, world: Rc<World>, pipeline: impl RenderPipeline + 'static) {
        if let Some(pipelines) = self.worlds.get_mut(&world) {
            pipelines.push(Box::new(pipeline));
        }
    }

    /// Runs the app main loop, this includes staring the window's event loop
    pub fn run(mut self) {
        self.window.run_event_loop(self.input_handler.clone(), || {
            if self.worlds.len() == 0 {
                return;
            }

            if let Some(active_world) = &self.active_world {
                if let Some(pipelines) = self.worlds.get(active_world) {
                    if let Some(command_buffer) = self.renderer.begin_frame() {
                        self.renderer.begin_swapchain_render_pass(command_buffer);

                        for pipeline in pipelines.iter() {
                            pipeline.draw(command_buffer);
                        }

                        self.renderer.end_swapchain_render_pass(command_buffer);
                        self.renderer.end_frame();
                    }
                }
            }
        });

        self.renderer.wait_device_idle();
    }
}
