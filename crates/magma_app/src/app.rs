use std::{cell::RefCell, path::Path, rc::Rc};

use magma_entities::prelude::{Camera, World};
use magma_input::prelude::InputHandler;
use magma_render::prelude::{Pipeline, PushConstantData, RenderPipeline, Renderer, Vertex};
use magma_window::prelude::Window;

/// Wraps a [`World`] with [`Pipeline`]s and a [`Camera`]
pub struct AppWorld {
    /// Actual world holding the entities
    world: World,
    /// Render pipelines which have been bound to this world
    pipelines: Vec<Box<dyn RenderPipeline>>,
    /// Camera used to render the world
    camera: Camera,
}

impl AppWorld {
    /// Creates a new [`AppWorld`] with no [`Pipeline`]s
    pub fn new(world: World, camera: Camera) -> AppWorld {
        AppWorld {
            world,
            pipelines: Vec::new(),
            camera,
        }
    }
}

/// Contains the application logic and data
///
/// Bundles together necessary features from other [`magma`] crates to create
/// a Vulkan application
pub struct App {
    /// The [`Window`] tied to this app
    window: Window,
    /// The Vulkan renderer that manages frames and synchronization with the GPU
    renderer: Renderer,
    /// All the [`AppWorld`]s this app has
    worlds: Vec<AppWorld>,
    /// The index of an [`AppWorld`] in [`worlds`] that will be updated and drawn each frame
    active_world: usize,
    /// Reference to the [`InputHandler`] collecting all input events from the [`window`]
    pub input_handler: Rc<RefCell<InputHandler>>,
}

impl App {
    /// Creates a new [`App`] with a default [`Window`] and [`Renderer`]
    pub fn new() -> App {
        // TODO: Allow the user to create a window using the builder so they can customize it
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

    /// Gets the aspect ratio of the frames being used by the [`Renderer`]
    pub fn aspect_ratio(&self) -> f32 {
        self.renderer.aspect_ratio()
    }

    /// Creates a new [`RenderPipeline`].
    ///
    /// See also [`Renderer::create_pipeline`]
    pub fn create_render_pipeline<P: 'static, V: 'static>(
        &mut self,
        vertex_shader: &Path,
        fragment_shader: &Path,
    ) -> Pipeline<P, V>
    where
        P: PushConstantData,
        V: Vertex,
    {
        // TODO: Allow for the push constant data to be bound to user-specified shaders
        self.renderer
            .create_pipeline::<P, V>(vertex_shader, fragment_shader)
    }

    /// Adds a new [`AppWorld`] to the app's worlds, returns the index of the inserted [`AppWorld`]
    pub fn add_world(&mut self, world: AppWorld) -> usize {
        self.worlds.push(world);
        self.worlds.len() - 1
    }

    /// Sets the index of the active [`AppWorld`].
    ///
    /// This world will be used in the next frame
    pub fn set_active_world(&mut self, world: usize) {
        self.active_world = world;
    }

    /// Adds a new [`RenderPipeline`] to an [`AppWorld`]
    pub fn add_render_pipeline(&mut self, world: usize, pipeline: impl RenderPipeline + 'static) {
        self.worlds
            .get_mut(world)
            .expect("Failed to get world at index")
            .pipelines
            .push(Box::new(pipeline));
    }

    /// Sets the [`Camera`] of an [`AppWorld`]
    pub fn set_world_camera(&mut self, world: usize, camera: Camera) {
        self.worlds
            .get_mut(world)
            .expect("Failed to get world at index")
            .camera = camera;
    }

    /// Runs the [`App`]s main loop, and starts the [`Window`]s event loop.
    /// 
    /// This is a blocking function call and won't finish until the [`Window`] exits its 
    /// event loop at which point the app will wait for the [`Renderer`] to finish with
    /// all tasks before returning.
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
