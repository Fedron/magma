use ash::vk;
use std::rc::Rc;

use crate::{core::device::LogicalDevice, pipeline::Pipeline, VulkanError};

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

#[derive(Copy, Clone, Debug)]
pub enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Pending,
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

pub enum CommandBufferLevel {
    Primary,
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

pub struct CommandBuffer {
    recording: bool,
    started_render_pass: bool,
    clear_color: (f32, f32, f32),

    handle: vk::CommandBuffer,
    device: Rc<LogicalDevice>,
}

impl CommandBuffer {
    pub fn new(handle: vk::CommandBuffer, device: Rc<LogicalDevice>) -> CommandBuffer {
        CommandBuffer {
            recording: false,
            started_render_pass: false,
            clear_color: (1.0, 0.0, 1.0),

            handle,
            device,
        }
    }
}

impl CommandBuffer {
    pub fn vk_handle(&self) -> vk::CommandBuffer {
        self.handle
    }
}

impl CommandBuffer {
    pub fn begin(&mut self) -> Result<(), CommandBufferError> {
        if self.recording {
            return Err(CommandBufferError::IncorrectState(
                CommandBufferState::Initial,
            ));
        }

        // TODO: Allow user to set the flags
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            self.device
                .vk_handle()
                .begin_command_buffer(self.handle, &begin_info)
                .map_err(|err| CommandBufferError::DeviceError(err.into()))?;
        };
        self.recording = true;

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), CommandBufferError> {
        if !self.recording {
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

        Ok(())
    }

    pub fn set_clear_color(&mut self, color: (f32, f32, f32)) {
        self.clear_color.0 = color.0.clamp(0.0, 1.0);
        self.clear_color.1 = color.1.clamp(0.0, 1.0);
        self.clear_color.2 = color.2.clamp(0.0, 1.0);
    }

    // TODO: Wrap framebuffer to include an extent
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

    pub fn end_render_pass(&mut self) {
        if self.started_render_pass {
            unsafe {
                self.device.vk_handle().cmd_end_render_pass(self.handle);
            };
            self.started_render_pass = false;
        }
    }

    pub fn bind_pipeline(&mut self, pipeline: &Pipeline) {
        unsafe {
            self.device.vk_handle().cmd_bind_pipeline(
                self.handle,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.vk_handle(),
            )
        };
    }

    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.device.vk_handle().cmd_draw(
                self.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        };
    }
}
