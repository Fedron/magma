use std::{cell::RefCell, path::Path, rc::Rc};

use magma_entities::prelude::{Camera, World};
use magma_input::prelude::InputHandler;
use magma_render::prelude::{Pipeline, PushConstantData, RenderPipeline, Renderer, Vertex};
use magma_window::prelude::Window;

/// Wraps a world from magma_entities with render pipelines and a camera
pub struct AppWorld {
    world: World,
    pipelines: Vec<Box<dyn RenderPipeline>>,
    camera: Camera,
}

impl AppWorld {
    pub fn new(world: World, camera: Camera) -> AppWorld {
        AppWorld {
            world,
            pipelines: Vec::new(),
            camera,
        }
    }
}

/// Represents an application that can draw to a window
pub struct App {
    window: Window,
    renderer: Renderer,
    worlds: Vec<AppWorld>,
    active_world: usize,
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
            worlds: Vec::new(),
            active_world: 0,
            input_handler: Rc::new(RefCell::new(InputHandler::new())),
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.renderer.aspect_ratio()
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

    pub fn add_world(&mut self, world: AppWorld) -> usize {
        self.worlds.push(world);
        self.worlds.len() - 1
    }

    pub fn set_active_world(&mut self, world: usize) {
        self.active_world = world;
    }

    pub fn add_render_pipeline(&mut self, world: usize, pipeline: impl RenderPipeline + 'static) {
        self.worlds
            .get_mut(world)
            .expect("Failed to get world at index")
            .pipelines
            .push(Box::new(pipeline));
    }

    pub fn set_world_camera(&mut self, world: usize, camera: Camera) {
        self.worlds
            .get_mut(world)
            .expect("Failed to get world at index")
            .camera = camera;
    }

    /// Runs the app main loop, this includes staring the window's event loop
    pub fn run(mut self) {
        self.window.run_event_loop(self.input_handler.clone(), || {
            if self.worlds.len() == 0 {
                return;
            }

            if let Some(active_world) = self.worlds.get_mut(self.active_world) {
                active_world.world.update();
                active_world.world.draw();

                if let Some(command_buffer) = self.renderer.begin_frame() {
                    self.renderer.begin_swapchain_render_pass(command_buffer);

                    for pipeline in active_world.pipelines.iter() {
                        pipeline.draw(command_buffer);
                    }

                    self.renderer.end_swapchain_render_pass(command_buffer);
                    self.renderer.end_frame();
                }
            }
        });

        self.renderer.wait_device_idle();
    }
}
