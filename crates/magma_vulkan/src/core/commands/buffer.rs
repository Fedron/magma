use ash::vk;
use std::rc::Rc;

use crate::{
    buffer::{Buffer, BufferUsageFlags},
    core::device::LogicalDevice,
    pipeline::{ubo::UniformBuffer, vertex::Vertex, Pipeline},
    VulkanError,
};

/// Errors that can be thrown by the CommandBuffer
#[derive(thiserror::Error, Debug)]
pub enum CommandBufferError {
    #[error("The command buffer is in an incorrect state, should be in the {0} state")]
    IncorrectState(CommandBufferState),
    #[error("A {0} command that was started was never ended prior to finishing recording the command buffer")]
    UnfinishedCommand(&'static str),
    #[error(
        "A {0} command is already started, end that one before starting another of the same type"
    )]
    CommandAlreadyStarted(&'static str),
    #[error(transparent)]
    DeviceError(VulkanError),
}

/// Represent the current lifecyle stage the command buffer is in
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommandBufferState {
    /// Command buffer was just allocated, or reset
    Initial,
    /// `begin()` was called on the command buffer
    Recording,
    /// The command buffer was recorded to and can be submitted to a [LogicalDevice]
    Executable,
    /// The command buffer was submitted to a queue and is awaiting execution
    Pending,
    /// A command result in setting the state of the Command buffer to invalid
    Invalid,
}

impl std::fmt::Display for CommandBufferState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandBufferState::Initial => write!(f, "Initial"),
            CommandBufferState::Recording => write!(f, "Recording"),
            CommandBufferState::Executable => write!(f, "Executable"),
            CommandBufferState::Pending => write!(f, "Pending"),
            CommandBufferState::Invalid => write!(f, "Invalid"),
        }
    }
}

/// Represents the level of a [CommandBuffer]
pub enum CommandBufferLevel {
    /// Can execute [CommandBufferLevel::Secondary] command buffers and be submitted to a
    /// [QueueHandle].
    Primary,
    /// Can be executed by [CommandBufferLevel::Primary] command buffers, but cannot be submitted
    /// to a [QueueHandle] directly.
    Secondary,
}

impl Into<vk::CommandBufferLevel> for CommandBufferLevel {
    fn into(self) -> vk::CommandBufferLevel {
        match self {
            CommandBufferLevel::Primary => vk::CommandBufferLevel::PRIMARY,
            CommandBufferLevel::Secondary => vk::CommandBufferLevel::SECONDARY,
        }
    }
}

/// Wraps a Vulkan command buffer
pub struct CommandBuffer {
    /// Which state the command buffer is in currently, determines what the command buffer can be
    /// used for and what methods can be used
    state: CommandBufferState,
    /// Whether a render pass was started on the command buffer
    started_render_pass: bool,
    /// The color to clear to
    clear_color: (f32, f32, f32),
    /// Vulkan handle of the currently bound graphics pipeline
    current_pipeline: Option<vk::Pipeline>,

    /// Opaque handle to Vulkan command buffer
    handle: vk::CommandBuffer,
    /// [LogicalDevice] the command buffer belongs to
    device: Rc<LogicalDevice>,
}

impl PartialEq for CommandBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}

impl CommandBuffer {
    /// Creates a new [CommandBuffer]
    pub fn new(handle: vk::CommandBuffer, device: Rc<LogicalDevice>) -> CommandBuffer {
        CommandBuffer {
            state: CommandBufferState::Initial,
            started_render_pass: false,
            clear_color: (1.0, 0.0, 1.0),
            current_pipeline: None,

            handle,
            device,
        }
    }
}

impl CommandBuffer {
    /// Returns the Vulkan handle to the command buffer
    pub(crate) fn vk_handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    /// Returns the Vulkan handle to the graphics pipeline that was most previously bound
    pub fn currently_bound_pipeline(&self) -> Option<vk::Pipeline> {
        self.current_pipeline
    }
}

impl CommandBuffer {
    /// Begins recording a command buffer, transitioning it into the
    /// [CommandBufferState::Recording] state.
    pub fn begin(&mut self) -> Result<(), CommandBufferError> {
        if !(self.state == CommandBufferState::Initial
            || self.state == CommandBufferState::Invalid
            || self.state == CommandBufferState::Executable)
        {
            return Err(CommandBufferError::IncorrectState(
                CommandBufferState::Initial,
            ));
        }

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .vk_handle()
                .begin_command_buffer(self.handle, &begin_info)
                .map_err(|err| CommandBufferError::DeviceError(err.into()))?;
        };
        self.state = CommandBufferState::Recording;

        Ok(())
    }

    /// Finishes recording the command buffer, transitioning the command buffer to the
    /// [CommandBufferState::Executable] state.
    pub fn end(&mut self) -> Result<(), CommandBufferError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferError::IncorrectState(
                CommandBufferState::Recording,
            ));
        }

        if self.started_render_pass {
            return Err(CommandBufferError::UnfinishedCommand("render pass"));
        }

        unsafe {
            self.device
                .vk_handle()
                .end_command_buffer(self.handle)
                .map_err(|err| CommandBufferError::DeviceError(err.into()))?;
        };
        self.state = CommandBufferState::Executable;

        Ok(())
    }

    /// Sets the clear color to use in the next render pass that is begun
    pub fn set_clear_color(&mut self, color: (f32, f32, f32)) {
        self.clear_color.0 = color.0.clamp(0.0, 1.0);
        self.clear_color.1 = color.1.clamp(0.0, 1.0);
        self.clear_color.2 = color.2.clamp(0.0, 1.0);
    }

    /// Sets the Vulkan viewport, will have an depth of 0-1 and be positioned at (0,0)
    pub fn set_viewport(&mut self, width: f32, height: f32) -> Result<(), CommandBufferError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferError::IncorrectState(
                CommandBufferState::Recording,
            ));
        }

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        unsafe {
            self.device
                .vk_handle()
                .cmd_set_viewport(self.handle, 0, &viewports);
        };

        Ok(())
    }

    /// Sets the Vulkan scissor, will have an offset of (0, 0)
    pub fn set_scissor(&mut self, extent: (u32, u32)) -> Result<(), CommandBufferError> {
        if self.state != CommandBufferState::Recording {
            return Err(CommandBufferError::IncorrectState(
                CommandBufferState::Recording,
            ));
        }

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: extent.0,
                height: extent.1,
            },
        }];

        unsafe {
            self.device
                .vk_handle()
                .cmd_set_scissor(self.handle, 0, &scissors);
        };

        Ok(())
    }

    /// Begins a render pass on the command buffer.
    ///
    /// The render pass will clear the framebuffer to the clear color of the [CommandBuffer]. The
    /// depth stencil attachment will also be cleared
    pub fn begin_render_pass(
        &mut self,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        extent: (u32, u32),
    ) -> Result<(), CommandBufferError> {
        if self.started_render_pass {
            return Err(CommandBufferError::CommandAlreadyStarted("render pass"));
        }

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [
                        self.clear_color.0,
                        self.clear_color.1,
                        self.clear_color.2,
                        1.0,
                    ],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: extent.0,
                    height: extent.1,
                },
            })
            .clear_values(&clear_values);

        unsafe {
            self.device.vk_handle().cmd_begin_render_pass(
                self.handle,
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        };

        self.started_render_pass = true;
        Ok(())
    }

    /// Ends a render pass on the [CommandBuffer]
    pub fn end_render_pass(&mut self) {
        if self.started_render_pass {
            unsafe {
                self.device.vk_handle().cmd_end_render_pass(self.handle);
            };
            self.started_render_pass = false;
        }
    }

    /// Binds a graphics pipeline
    pub fn bind_pipeline<V, P>(&mut self, pipeline: &Pipeline<V, P>)
    where
        V: Vertex,
        P: UniformBuffer,
    {
        unsafe {
            self.device.vk_handle().cmd_bind_pipeline(
                self.handle,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.vk_handle(),
            )
        };
        self.current_pipeline = Some(pipeline.vk_handle());
    }

    /// Binds a vertex buffer
    pub unsafe fn bind_vertex_buffer<T, const CAPACITY: usize>(
        &mut self,
        buffer: &Buffer<T, CAPACITY>,
    ) {
        if !buffer.usage().contains(BufferUsageFlags::VERTEX_BUFFER) {
            log::warn!(
                "Can't bind a buffer as vertex buffer if it's usage hasn't been marked as such"
            );
            return;
        }

        let buffers = [buffer.vk_handle()];
        let offsets = [0];

        self.device
            .vk_handle()
            .cmd_bind_vertex_buffers(self.handle, 0, &buffers, &offsets);
    }

    /// Binds an index buffer
    pub unsafe fn bind_index_buffer<const CAPACITY: usize>(
        &mut self,
        buffer: &Buffer<u32, CAPACITY>,
    ) {
        if !buffer.usage().contains(BufferUsageFlags::INDEX_BUFFER) {
            log::warn!(
                "Can't bind a buffer as an index buffer it it's usage hasn't been marked as such"
            );
            return;
        }

        self.device.vk_handle().cmd_bind_index_buffer(
            self.handle,
            buffer.vk_handle(),
            0,
            vk::IndexType::UINT32,
        );
    }

    /// Draws from the vertices in the previoulsy bound vertex buffer
    pub unsafe fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
            self.device.vk_handle().cmd_draw(
                self.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
    }

    /// Draws from a vertex buffer using an index buffer as well
    pub unsafe fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: u32,
        first_instance: u32,
    ) {
            self.device.vk_handle().cmd_draw_indexed(
                self.handle,
                index_count,
                instance_count,
                first_index,
                vertex_offset as i32,
                first_instance,
            );
    }
}
