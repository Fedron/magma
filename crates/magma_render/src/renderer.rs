use ash::vk;
use std::{path::Path, rc::Rc};
use winit::window::Window;

use crate::{
    device::Device,
    pipeline::{Pipeline, PipelineConfigInfo},
    swapchain::Swapchain,
};

/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputAttributeDescription.html
pub type VertexAttributeDescription = vk::VertexInputAttributeDescription;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputBindingDescription.html
pub type VertexBindingDescription = vk::VertexInputBindingDescription;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputRate.html
pub type VertexInputRate = vk::VertexInputRate;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html
pub type Format = vk::Format;

/// Represents a vertex that is passed to a shader.
///
/// Allows for a struct to be passed to a [`RenderPipeline`] by providing descriptions
/// for every field in the struct.
pub trait Vertex {
    /// Returns attribute descriptions for each field in the struct.
    ///
    /// The attribute descriptions should match the layout in the vertex shader
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
    /// Returns the binding descriptions for the struct.
    ///
    /// The binding descriptions should match the layout in the vertex shader
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
}

/// Allows for a struct to be passed to a [`RenderPipeline`] as a Vulkan push constant
pub trait PushConstantData {
    /// Converts [`PushConstantData`] to an array of bytes
    fn as_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            let size_in_bytes = std::mem::size_of::<Self>();
            let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
            std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
        }
    }
}

/// Contains all the logic and data needed to get Vulkan to render to a window
pub struct Renderer {
    /// Handle to a winit [`Window`][winit::window::Window] that is drawn to
    window: Rc<Window>,
    /// Reference to a [`Device`] that the renderer uses
    device: Rc<Device>,
    /// Current [`Swapchain`] being used
    swapchain: Swapchain,
    /// Vulkan [`CommandBuffer`][ash::vk::CommandBuffer]s for each framebuffer in the [`Renderer::swapchain`]
    command_buffers: Vec<vk::CommandBuffer>,
    /// Index of the current image and framebuffer being drawn
    ///
    /// Assigned when [`Renderer::begin_frame`] is called
    current_image_index: usize,
    /// Indicates whether a frame is in progress of being drawn
    ///
    /// - Set to true when [`Renderer::begin_frame`] is called
    /// - Set to false when [`Renderer::end_frame`] is called
    is_frame_started: bool,
    /// Color the framebuffer is reset to when a render pass is started using [`Renderer::begin_swapchain_render_pass`]
    clear_color: [f32; 4],
}

impl Renderer {
    /// Creates a new [`Renderer`] that targets the given window
    ///
    /// The first dedicated GPU found is chosen as the device for this [`Renderer`]
    /// and a double-buffered [`Swapchain`] is created for the [`Renderer`] with the
    /// current extent of the window.
    pub fn new(window: Rc<Window>, clear_color: [f32; 4]) -> Renderer {
        let device = Rc::new(Device::new(window.as_ref()));
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
            clear_color,
        }
    }

    /// Creates a new [`RenderPipeline`]
    ///
    /// The created [`Pipeline`] will create a layout with the [`PushConstantData`] provided
    /// and bind the [`Vertex`] attributes in the pipeline. The [`Pipeline`] will be created
    /// using the [`Device`] the [`Renderer`] is currently using and the render pass of the
    /// current [`Swapchain`] being used by the [`Renderer`]. The [`Pipeline`] is not recreated
    /// when the [`Swapchain`] is, it is assumed they will be compatible.
    ///
    /// The [`Pipeline`] will create two Vulkan shader modules, one for your vertex shader and
    /// the other for the fragment shader. It is expected that the [`PushConstantData`] and
    /// [`Vertex`] match your shaders, this is not checked and is up to you.
    ///
    /// [`PushConstantData`] is only bound to the vertex shader.
    pub fn create_pipeline<P: 'static, V: 'static>(
        &mut self,
        vertex_shader: &Path,
        fragment_shader: &Path,
    ) -> Pipeline<P, V>
    where
        P: PushConstantData,
        V: Vertex,
    {
        let config = PipelineConfigInfo::default();

        Pipeline::new(
            self.device.clone(),
            config,
            &self.swapchain.render_pass,
            vertex_shader,
            fragment_shader,
        )
    }

    /// Creates a new [`Swapchain`], using the current [`Swapchain`] of the [`Renderer`] as a base,
    /// to match the new [`Window`][winit::window::Window] size.
    pub fn recreate_swapchain(&mut self) {
        // Wait until the device is finished with the current swapchain before recreating it
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

    /// Gets the aspect ratio of the current [`Swapchain`]
    pub fn aspect_ratio(&self) -> f32 {
        self.swapchain.extent_aspect_ratio()
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
                .device
                .free_command_buffers(self.device.command_pool, &self.command_buffers);
        };
        self.command_buffers.clear();
    }

    /// Gets the render pass being used by the current [`Swapchain`]
    pub fn get_swapchain_render_pass(&self) -> vk::RenderPass {
        self.swapchain.render_pass
    }

    /// Begins a new render pass using the render pass of the current [`Swapchain`].
    ///
    /// Before calling this, it is required that a frame has been started and the command buffer
    /// matches the command buffer being used for that frame.
    ///
    /// The screen will be cleared to a light gray, and the viewport and scissor will be updated
    /// with the extent of the current [`Swapchain`]
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

    /// Ends an existing render pass of the render pass of the current [`Swapchain`].
    ///
    /// Before calling this it is required that a frame has been started and the command buffer
    /// matches the command buffer being used for that frame.
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

    /// Begins a new frame, returning the [`CommandBuffer`][ash::vk::CommandBuffer] that will be
    /// used to draw that frame.
    ///
    /// If a frame has already been started then the [`Renderer`] will panic.
    ///
    /// Acquires the next image to draw to from the current [`Swapchain`]. If the [`Swapchain`]
    /// is suboptimal or out of date, the [`Swapchain`] will be recreated and no command buffer
    /// will be returned.
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

    /// Ends the frame submitting the command buffer and causing a draw to the window.
    ///
    /// A frame should have already been begun prior to this function being called, if not the
    /// [`Renderer`] will panic.
    ///
    /// If at any point the [`Swapchain`] comes back as being suboptimal or out of date then it
    /// will be recreated and the frame ended.
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

    /// Returns the [`CommandBuffer`][ash::vk::CommandBuffer] that is currently being used
    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        if !self.is_frame_started {
            log::error!("Cannot get a command buffer when a frame is not in progress");
            panic!("Failed to get command buffer, see above");
        }
        self.command_buffers[self.current_image_index]
    }

    /// Waits for the [`Device`] in this [`Renderer`] to idle
    pub fn wait_device_idle(&self) {
        unsafe {
            self.device
                .device
                .device_wait_idle()
                .expect("Failed to wait for device to idle");
        }
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
