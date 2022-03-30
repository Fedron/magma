use std::rc::Rc;

use ash::vk;
use glam::{vec3, Vec3};

use crate::{
    components::Camera,
    device::Device,
    mesh::{SimplePush, Vertex},
    pipeline::{Pipeline, PipelineConfigInfo, PushConstant, PushConstantData, Shader},
    prelude::Window,
    renderable::Renderable,
    swapchain::Swapchain,
};

pub struct Engine {
    window: Window,
    device: Rc<Device>,
    swapchain: Swapchain,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: usize,
    is_frame_started: bool,
    clear_color: [f32; 4],

    renderables: Vec<Renderable>,
    camera: Camera,
}

impl Engine {
    pub fn new(window: Window, clear_color: [f32; 4]) -> Engine {
        let device = Rc::new(Device::new(&window.winit()));
        let swapchain = Swapchain::new(device.clone());
        let command_buffers = Engine::create_command_buffers(
            device.vk(),
            device.command_pool,
            swapchain.framebuffers.len() as u32,
        );

        let mut camera = Camera::new();
        camera.set_perspective(
            50_f32.to_radians(),
            swapchain.extent_aspect_ratio(),
            0.1,
            20.0,
        );
        camera.look_at(vec3(0.0, 2.5, -10.0), Vec3::ZERO);

        Engine {
            window,
            device,
            swapchain,
            command_buffers,
            current_image_index: 0,
            is_frame_started: false,
            clear_color,
            renderables: Vec::new(),
            camera,
        }
    }

    /// Creates empty Vulkan [`CommandBuffer`][ash::vk::CommandBuffer]s
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

    /// Frees all the [`CommandBuffer`][ash::vk::CommandBuffer]s being used by the [`Renderer`]
    fn free_command_buffers(&mut self) {
        unsafe {
            self.device
                .vk()
                .free_command_buffers(self.device.command_pool, &self.command_buffers);
        };
        self.command_buffers.clear();
    }

    /// Creates a new [`Swapchain`], using the current [`Swapchain`] of the [`Renderer`] as a base,
    /// to match the new [`Window`][winit::window::Window] size.
    fn recreate_swapchain(&mut self) {
        // Wait until the device is finished with the current swapchain before recreating it
        unsafe {
            self.device
                .vk()
                .device_wait_idle()
                .expect("Failed to wait for GPU to idle");
        };

        let window_size = self.window.winit().inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        // Recreate swapchain
        self.swapchain =
            Swapchain::from_old_swapchain(self.device.clone(), self.swapchain.swapchain);
        if self.swapchain.framebuffers.len() != self.command_buffers.len() {
            self.free_command_buffers();
            self.command_buffers = Engine::create_command_buffers(
                &self.device.vk(),
                self.device.command_pool,
                self.swapchain.framebuffers.len() as u32,
            );
        }
    }

    /// Begins a new render pass using the render pass of the current [`Swapchain`].
    ///
    /// Before calling this, it is required that a frame has been started and the command buffer
    /// matches the command buffer being used for that frame.
    ///
    /// The screen will be cleared to a light gray, and the viewport and scissor will be updated
    /// with the extent of the current [`Swapchain`]
    fn begin_swapchain_render_pass(&self, command_buffer: vk::CommandBuffer) {
        if !self.is_frame_started {
            log::error!("Cannot begin a swapchain render pass if no frame is in progress");
            panic!("Failed to begin swapchain render pass, see above");
        }

        if command_buffer != self.get_current_command_buffer() {
            log::error!("Cannot begin a swapchain render pass on a command buffer that belongs to a different frame");
            panic!("Failed to begin swapchain render pass, see above");
        }

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: self.clear_color,
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.swapchain.render_pass)
            .framebuffer(self.swapchain.framebuffers[self.current_image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.vk().cmd_begin_render_pass(
                command_buffer,
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
                .vk()
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.device
                .vk()
                .cmd_set_scissor(command_buffer, 0, &scissors);
        }
    }

    /// Ends an existing render pass of the render pass of the current [`Swapchain`].
    ///
    /// Before calling this it is required that a frame has been started and the command buffer
    /// matches the command buffer being used for that frame.
    fn end_swapchain_render_pass(&self, command_buffer: vk::CommandBuffer) {
        if !self.is_frame_started {
            log::error!("Cannot end a swapchain render pass if no frame is in progress");
            panic!("Failed to end swapchain render pass, see above");
        }

        if command_buffer != self.get_current_command_buffer() {
            log::error!("Cannot end a swapchain render pass on a command buffer that belongs to a different frame");
            panic!("Failed to end swapchain render pass, see above");
        }

        unsafe {
            self.device.vk().cmd_end_render_pass(command_buffer);
        };
    }

    /// Begins a new frame, returning the [`CommandBuffer`][ash::vk::CommandBuffer] that will be
    /// used to draw that frame.
    ///
    /// If a frame has already been started then the [`Renderer`] will panic.
    ///
    /// Acquires the next image to draw to from the current [`Swapchain`]. If the [`Swapchain`]
    /// is suboptimal or out of date, the [`Swapchain`] will be recreated and no command buffer
    /// will be returned.
    fn begin_frame(&mut self) -> Option<vk::CommandBuffer> {
        if self.is_frame_started {
            log::error!("Cannot begin a new frame, while another is already in progress");
            panic!("Failed to begin frame, see above");
        }

        let result = self.swapchain.acquire_next_image();
        if result.is_err() {
            self.recreate_swapchain();
            return None;
        }

        let (image_index, is_sub_optimal) = result.unwrap();
        if is_sub_optimal {
            self.recreate_swapchain();
            return None;
        }

        self.current_image_index = image_index as usize;
        self.is_frame_started = true;
        let command_buffer = self.get_current_command_buffer();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .vk()
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording to command buffer")
        };

        Some(command_buffer)
    }

    /// Ends the frame submitting the command buffer and causing a draw to the window.
    ///
    /// A frame should have already been begun prior to this function being called, if not the
    /// [`Renderer`] will panic.
    ///
    /// If at any point the [`Swapchain`] comes back as being suboptimal or out of date then it
    /// will be recreated and the frame ended.
    fn end_frame(&mut self) {
        if !self.is_frame_started {
            log::error!("Cannot end an frame when no frame has been started");
            panic!("Failed to end frame, see above");
        }

        let command_buffer = self.get_current_command_buffer();
        unsafe {
            self.device
                .vk()
                .end_command_buffer(command_buffer)
                .expect("Failed to finish recording command buffer");
        };

        let is_sub_optimal = self
            .swapchain
            .submit_command_buffers(command_buffer, self.current_image_index);

        if is_sub_optimal.is_err() {
            self.recreate_swapchain();
        } else if is_sub_optimal.unwrap() {
            self.recreate_swapchain();
        }

        self.is_frame_started = false;
    }

    /// Returns the [`CommandBuffer`][ash::vk::CommandBuffer] that is currently being used
    fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        if !self.is_frame_started {
            log::error!("Cannot get a command buffer when a frame is not in progress");
            panic!("Failed to get command buffer, see above");
        }
        self.command_buffers[self.current_image_index]
    }
}

impl Engine {
    pub fn device(&self) -> Rc<Device> {
        self.device.clone()
    }

    pub fn create_pipeline<V>(&self, shaders: &[Shader]) -> Pipeline
    where
        V: Vertex,
    {
        Pipeline::new::<V>(
            self.device.clone(),
            PipelineConfigInfo::default(),
            &self.swapchain.render_pass,
            shaders,
            &[PushConstant {
                stage: Shader::VERTEX,
                offset: 0,
                size: std::mem::size_of::<SimplePush>(),
            }],
        )
    }

    pub fn add_renderable(&mut self, renderable: Renderable) {
        self.renderables.push(renderable);
    }

    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.window.poll_events();

            if let Some(command_buffer) = self.begin_frame() {
                self.begin_swapchain_render_pass(command_buffer);

                if self.renderables.len() > 0 {
                    let mut first_iter = true;
                    let mut current_pipeline =
                        self.renderables.first().unwrap().pipeline.graphics_pipeline;
                    for renderable in self.renderables.iter() {
                        if current_pipeline != renderable.pipeline.graphics_pipeline || first_iter {
                            unsafe {
                                self.device.vk().cmd_bind_pipeline(
                                    command_buffer,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    renderable.pipeline.graphics_pipeline,
                                );
                            };
                            current_pipeline = renderable.pipeline.graphics_pipeline;
                            first_iter = false;
                        }

                        let buffers = [renderable.mesh.vertex_buffer.vk()];
                        let offsets = [0];

                        unsafe {
                            self.device.vk().cmd_bind_vertex_buffers(
                                command_buffer,
                                0,
                                &buffers,
                                &offsets,
                            );
                            self.device.vk().cmd_bind_index_buffer(
                                command_buffer,
                                renderable.mesh.indices_buffer.vk(),
                                0,
                                vk::IndexType::UINT32,
                            );

                            let push_constant = SimplePush {
                                transform: self.camera.projection_matrix()
                                    * self.camera.view_matrix()
                                    * renderable.transform.as_matrix(),
                            };
                            self.device.vk().cmd_push_constants(
                                command_buffer,
                                renderable.pipeline.layout,
                                vk::ShaderStageFlags::VERTEX,
                                0,
                                push_constant.as_bytes(),
                            );

                            self.device.vk().cmd_draw_indexed(
                                command_buffer,
                                renderable.mesh.indices_buffer.len() as u32,
                                1,
                                0,
                                0,
                                0,
                            );
                        };
                    }
                }

                self.end_swapchain_render_pass(command_buffer);
                self.end_frame();
            }
        }

        unsafe {
            self.device
                .vk()
                .device_wait_idle()
                .expect("Failed to wait for GPU to idle");
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.free_command_buffers();
    }
}
