use ash::vk;

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
    handle: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn vk_handle(&self) -> vk::CommandBuffer {
        self.handle
    }
}

impl From<vk::CommandBuffer> for CommandBuffer {
    fn from(handle: vk::CommandBuffer) -> Self {
        CommandBuffer { handle }
    }
}
