use ash::vk;
use bitflags::bitflags;
use std::rc::Rc;

use crate::prelude::LogicalDevice;

bitflags! {
    pub struct CommandPoolFlags: u32 {
        const TRANSIENT = 0b1;
        const RESETTABLE = 0b10;
        const PROTECTED = 0b100;
    }
}

impl Into<vk::CommandPoolCreateFlags> for CommandPoolFlags {
    fn into(self) -> vk::CommandPoolCreateFlags {
        vk::CommandPoolCreateFlags::from_raw(self.bits)
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

pub struct CommandPool {
    device: Rc<LogicalDevice>,

    buffers: Vec<CommandBuffer>,
    handle: vk::CommandPool,
}

impl CommandPool {
    pub fn new(
        device: Rc<LogicalDevice>,
        flags: CommandPoolFlags,
        queue_family_index: u32,
    ) -> CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(flags.into())
            .queue_family_index(queue_family_index);

        let handle = unsafe {
            device
                .vk_handle()
                .create_command_pool(&create_info, None)
                .expect("Failed to create command pool")
        };

        CommandPool {
            device,
            buffers: Vec::new(),
            handle,
        }
    }
}

impl CommandPool {
    pub fn vk_handle(&self) -> vk::CommandPool {
        self.handle
    }
}

impl CommandPool {
    pub fn allocate_buffers(&mut self, count: u32, level: CommandBufferLevel) {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(count)
            .command_pool(self.handle)
            .level(level.into());

        let vk_buffers = unsafe {
            self.device
                .vk_handle()
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers")
        };
        let mut buffers = vk_buffers
            .iter()
            .map(|&buffer| CommandBuffer::new(self.device.clone(), buffer))
            .collect();

        self.buffers.append(&mut buffers);
    }

    pub fn free_buffers(&mut self) {
        let vk_buffers: Vec<vk::CommandBuffer> =
            self.buffers.iter().map(|buffer| buffer.handle).collect();

        unsafe {
            self.device
                .vk_handle()
                .free_command_buffers(self.handle, &vk_buffers);
        };
        self.buffers.clear();
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        self.free_buffers();
        unsafe {
            self.device
                .vk_handle()
                .destroy_command_pool(self.handle, None);
        }
    }
}

bitflags! {
    pub struct CommandBufferUsageFlags: u32 {
        const ONE_TIME = 0b1;
        const RENDER_PASS = 0b10;
        const SIMULTANEOUS = 0b100;
    }
}

impl Into<vk::CommandBufferUsageFlags> for CommandBufferUsageFlags {
    fn into(self) -> vk::CommandBufferUsageFlags {
        vk::CommandBufferUsageFlags::from_raw(self.bits)
    }
}

pub enum PipelineBindPoint {
    Graphics,
    Compute,
}

impl Into<vk::PipelineBindPoint> for PipelineBindPoint {
    fn into(self) -> vk::PipelineBindPoint {
        match self {
            PipelineBindPoint::Graphics => vk::PipelineBindPoint::GRAPHICS,
            PipelineBindPoint::Compute => vk::PipelineBindPoint::COMPUTE,
        }
    }
}

pub struct CommandBuffer {
    is_recording: bool,
    use_render_pass: bool,
    clear_color: (f32, f32, f32),

    device: Rc<LogicalDevice>,
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(device: Rc<LogicalDevice>, vk_handle: vk::CommandBuffer) -> CommandBuffer {
        CommandBuffer {
            is_recording: false,
            use_render_pass: false,
            clear_color: (1.0, 0.0, 1.0),

            device,
            handle: vk_handle,
        }
    }
}

impl CommandBuffer {
    pub fn start_recording(&mut self, usage: CommandBufferUsageFlags) {
        if self.is_recording {
            panic!("Buffer is already recording, cannot start recording again. Please finish recording first");
        }
        self.is_recording = true;

        let begin_info = vk::CommandBufferBeginInfo::builder().flags(usage.into());
        unsafe {
            self.device
                .vk_handle()
                .begin_command_buffer(self.handle, &begin_info)
                .expect("Failed to begin the command buffer");
        };
    }

    pub fn set_clear_color(&mut self, r: f32, g: f32, b: f32) {
        self.clear_color = (r, g, b);
    }

    pub fn use_render_pass(
        &mut self,
        render_pass: vk::RenderPass,
        framebuffer: vk::Framebuffer,
        render_area: (u32, u32),
    ) {
        self.use_render_pass = true;

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [
                    self.clear_color.0,
                    self.clear_color.1,
                    self.clear_color.2,
                    1.0,
                ],
            },
        }];

        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: render_area.0,
                    height: render_area.1,
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
    }

    pub fn bind_pipeline(&mut self, pipeline: vk::Pipeline, bind_point: PipelineBindPoint) {
        unsafe {
            self.device
                .vk_handle()
                .cmd_bind_pipeline(self.handle, bind_point.into(), pipeline)
        };
    }

    pub fn bind_vertex_buffer(&mut self, buffer: vk::Buffer) {
        let buffers = [buffer];
        let offsets = [0];

        unsafe {
            self.device
                .vk_handle()
                .cmd_bind_vertex_buffers(self.handle, 0, &buffers, &offsets);
        };
    }

    pub fn end_recording(&mut self) {
        if !self.is_recording {
            panic!("Cannot end recording a buffer that never started recording");
        }
        self.is_recording = false;

        if self.use_render_pass {
            unsafe {
                self.device.vk_handle().cmd_end_render_pass(self.handle);
            };
        }

        unsafe {
            self.device
                .vk_handle()
                .end_command_buffer(self.handle)
                .expect("Failed to end the command buffer");
        };
    }
}
