use std::rc::Rc;

use ash::vk;

use crate::{device::Device, prelude::Window, swapchain::Swapchain};

pub struct Engine {
    window: Window,
    device: Rc<Device>,
    swapchain: Swapchain,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: usize,
    is_frame_started: bool,
    clear_color: [f32; 4],
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

        Engine {
            window,
            device,
            swapchain,
            command_buffers,
            current_image_index: 0,
            is_frame_started: false,
            clear_color,
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
    pub fn run(&mut self) {
        while !self.window.should_close() {
            self.window.poll_events();

            if let Some(command_buffer) = self.begin_frame() {
                self.begin_swapchain_render_pass(command_buffer);
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
