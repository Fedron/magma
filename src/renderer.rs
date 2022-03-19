use ash::vk;
use std::rc::Rc;
use winit::window::Window;

pub mod device;
pub mod pipeline;
pub mod simple_render_system;
pub mod swapchain;

use self::{device::Device, swapchain::Swapchain};

pub struct Renderer {
    /// Handle to the window that is being drawn to
    window: Rc<Window>,
    /// Handle to logical device
    device: Rc<Device>,
    /// Handle to currently active swapchain
    swapchain: Swapchain,
    /// List of all command buffers being used
    command_buffers: Vec<vk::CommandBuffer>,
    /// Index of the current framebuffer and command buffer being used
    current_image_index: usize,
    /// Indicates whether a frame has been started using 'begin_frame'
    is_frame_started: bool,
}

impl Renderer {
    /// Creates a new Renderer that will draw to the window provided
    pub fn new(window: Rc<Window>, device: Rc<Device>) -> Renderer {
        let swapchain = Swapchain::new(device.clone());
        let command_buffers = Renderer::create_command_buffers(
            &device.device,
            device.command_pool,
            swapchain.framebuffers.len() as u32,
        );

        Renderer {
            window,
            device,
            swapchain,
            command_buffers,
            current_image_index: 0,
            is_frame_started: false,
        }
    }

    /// Recreates the swapchain and graphics pipeline to match the new window size
    pub fn recreate_swapchain(&mut self) {
        // Wait until the device is finished with the current swapchain before recreating ti
        unsafe {
            self.device
                .device
                .device_wait_idle()
                .expect("Failed to wait for GPU to idle");
        };

        let window_size = self.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        // Recreate swapchain
        self.swapchain =
            Swapchain::from_old_swapchain(self.device.clone(), self.swapchain.swapchain);
        if self.swapchain.framebuffers.len() != self.command_buffers.len() {
            self.free_command_buffers();
            self.command_buffers = Renderer::create_command_buffers(
                &self.device.device,
                self.device.command_pool,
                self.swapchain.framebuffers.len() as u32,
            );
        }
    }

    /// Gets the aspect ratio of the swapchain
    pub fn aspect_ratio(&self) -> f32 {
        self.swapchain.extent_aspect_ratio()
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

    /// Returns the render pass being used by the swapchain
    pub fn get_swapchain_render_pass(&self) -> vk::RenderPass {
        self.swapchain.render_pass
    }

    /// Begins the swapchain's render pass
    pub fn begin_swapchain_render_pass(&self, command_buffer: vk::CommandBuffer) {
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
                    float32: [0.1, 0.1, 0.1, 1.0],
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
            self.device.device.cmd_begin_render_pass(
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
                .device
                .cmd_set_viewport(command_buffer, 0, &viewports);
            self.device
                .device
                .cmd_set_scissor(command_buffer, 0, &scissors);
        }
    }

    /// Ends the swapchain's render pass
    pub fn end_swapchain_render_pass(&self, command_buffer: vk::CommandBuffer) {
        if !self.is_frame_started {
            log::error!("Cannot end a swapchain render pass if no frame is in progress");
            panic!("Failed to end swapchain render pass, see above");
        }

        if command_buffer != self.get_current_command_buffer() {
            log::error!("Cannot end a swapchain render pass on a command buffer that belongs to a different frame");
            panic!("Failed to end swapchain render pass, see above");
        }

        unsafe {
            self.device.device.cmd_end_render_pass(command_buffer);
        };
    }

    /// Begins frame that can be drawn to, returns the command buffer to write commands to
    ///
    /// Acquires the next image to draw to from the swapchain and if the swapchain is suboptimal or out of date
    /// then the swapchain will recreated and the frame won't begin.
    pub fn begin_frame(&mut self) -> Option<vk::CommandBuffer> {
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
                .device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording to command buffer")
        };

        Some(command_buffer)
    }

    /// Ends the frame submitting the command buffer and causing a draw to the window
    ///
    /// If at any point the swapchain comes back as being suboptimal or out of date then it will be recreated
    /// and the frame ended
    pub fn end_frame(&mut self) {
        if !self.is_frame_started {
            log::error!("Cannot end an frame when no frame has been started");
            panic!("Failed to end frame, see above");
        }

        let command_buffer = self.get_current_command_buffer();
        unsafe {
            self.device
                .device
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

    /// Returns the command buffer that is currently being used
    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        if !self.is_frame_started {
            log::error!("Cannot get a command buffer when a frame is not in progress");
            panic!("Failed to get command buffer, see above");
        }
        self.command_buffers[self.current_image_index]
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.free_command_buffers();
            self.device
                .device
                .destroy_command_pool(self.device.command_pool, None);
        };
    }
}
