use ash::vk;
use std::rc::Rc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    entity::Entity,
    utils,
    vulkan::{
        device::Device,
        pipeline::{Align16, Pipeline, PushConstants},
        swapchain::Swapchain,
    },
};

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit window
    window: winit::window::Window,
    /// Handle to logical device
    pub device: Rc<Device>,
    /// Handle to currently active swapchain
    swapchain: Swapchain,
    /// Handle to the current graphics pipeline
    pipeline: Pipeline,
    /// List of all command buffers being used
    command_buffers: Vec<vk::CommandBuffer>,
    /// List of all entities in the 'world'
    entities: Vec<Entity>,
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let window = App::init_window(event_loop);
        let device = Rc::new(Device::new(&window));
        let swapchain = Swapchain::new(device.clone());

        let pipeline = Pipeline::new(device.clone(), swapchain.render_pass);

        let command_buffers = App::create_command_buffers(
            &device.device,
            device.command_pool,
            swapchain.framebuffers.len() as u32,
        );

        let window_size = window.inner_size();

        App {
            window,
            device,
            swapchain,
            pipeline,
            command_buffers,
            entities: Vec::new(),
            window_size,
        }
    }

    /// Recreates the swapchain and graphics pipeline to match the new window size
    fn recreate_swapchain(&mut self) {
        // Wait until the device is finished with the current swapchain before recreating ti
        unsafe {
            self.device
                .device
                .device_wait_idle()
                .expect("Failed to wait for GPU to idle");
        };

        if self.window_size.width == 0 || self.window_size.height == 0 {
            return;
        }

        // Recreate swapchain
        self.swapchain =
            Swapchain::from_old_swapchain(self.device.clone(), self.swapchain.swapchain);
        if self.swapchain.framebuffers.len() != self.command_buffers.len() {
            self.free_command_buffers();
            self.command_buffers = App::create_command_buffers(
                &self.device.device,
                self.device.command_pool,
                self.swapchain.framebuffers.len() as u32,
            );
        }

        self.pipeline = Pipeline::new(self.device.clone(), self.swapchain.render_pass);
    }

    /// Creates new Vulkan command buffers for every framebuffer
    ///
    /// Nothing is recorded into the command buffers
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        amount: u32,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(amount)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
        };

        command_buffers
    }

    /// Frees all the command buffers currently in the command pool
    fn free_command_buffers(&mut self) {
        unsafe {
            self.device
                .device
                .free_command_buffers(self.device.command_pool, &self.command_buffers);
        };
        self.command_buffers.clear();
    }

    /// Records commands for a command buffer
    ///
    /// The commands consist of creating a viewport, binding the pipeline, and drawing the model
    fn record_command_buffer(&mut self, index: usize) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .device
                .begin_command_buffer(self.command_buffers[index], &command_buffer_begin_info)
                .expect("Failed to begin recording to command buffer")
        };

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.1, 0.1, 0.1, 1.0],
            },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.swapchain.render_pass)
            .framebuffer(self.swapchain.framebuffers[index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.device.cmd_begin_render_pass(
                self.command_buffers[index],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            }];

            self.device
                .device
                .cmd_set_viewport(self.command_buffers[index], 0, &viewports);
            self.device
                .device
                .cmd_set_scissor(self.command_buffers[index], 0, &scissors);

            self.device.device.cmd_bind_pipeline(
                self.command_buffers[index],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipeline,
            );

            for entity in self.entities.iter_mut() {
                entity.model().bind(self.command_buffers[index]);
                entity.transform.rotation += 0.1;

                let push = PushConstants {
                    transform: Align16(entity.transform_matrix()),
                    translation: Align16(entity.transform.position),
                };

                self.device.device.cmd_push_constants(
                    self.command_buffers[index],
                    self.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push.as_bytes(),
                );

                entity.model().draw(self.command_buffers[index]);
            }

            self.device
                .device
                .cmd_end_render_pass(self.command_buffers[index]);
            self.device
                .device
                .end_command_buffer(self.command_buffers[index])
                .expect("Failed to finish recording command buffer");
        }
    }

    /// Initialises a winit window, returning the initialised window
    pub fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_title(utils::constants::WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                utils::constants::WINDOW_WIDTH,
                utils::constants::WINDOW_HEIGHT,
            ))
            .build(event_loop)
            .expect("")
    }

    /// Adds a new entity that will be rendered
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    self.window_size = size;
                    self.recreate_swapchain();
                }
                _ => {}
            },
            Event::MainEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => {
                let result = self.swapchain.acquire_next_image();
                if result.is_err() {
                    self.recreate_swapchain();
                    return;
                }

                let (image_index, is_sub_optimal) = result.unwrap();
                if is_sub_optimal {
                    self.recreate_swapchain();
                    return;
                }

                self.record_command_buffer(image_index as usize);
                let is_sub_optimal = self.swapchain.submit_command_buffers(
                    self.command_buffers[image_index as usize],
                    image_index as usize,
                );

                if is_sub_optimal.is_err() {
                    self.recreate_swapchain();
                    return;
                } else if is_sub_optimal.unwrap() {
                    self.recreate_swapchain();
                    return;
                }
            }
            Event::LoopDestroyed => {
                unsafe {
                    self.device
                        .device
                        .device_wait_idle()
                        .expect("Failed to wait until device idle");
                };
            }
            _ => {}
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_command_pool(self.device.command_pool, None);
        };
    }
}
