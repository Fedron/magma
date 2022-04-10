use ash::vk;
use std::rc::Rc;

use super::buffer::{CommandBuffer, CommandBufferLevel};
use crate::{
    core::device::{LogicalDevice, QueueFamily},
    VulkanError,
};

/// Errors that can be returned by the [CommandPool]
#[derive(thiserror::Error, Debug)]
pub enum CommandPoolError {
    #[error(transparent)]
    DeviceError(#[from] VulkanError),
}

/// Wraps a Vulkan command pool
pub struct CommandPool {
    /// [CommandBuffers][CommandBuffer] that are allocated to this [CommandPool]
    buffers: Vec<CommandBuffer>,
    /// Opaque handle to Vulkan command pool
    handle: vk::CommandPool,
    /// [LogicalDevice] this command pool belongs to
    device: Rc<LogicalDevice>,
}

impl CommandPool {
    /// Creates a new [CommandPool].
    ///
    /// Any [CommandBuffers][CommandBuffer] allocated from this [CommandPool] can only be submitted
    /// to queues in `queue_family`.
    pub fn new(
        device: Rc<LogicalDevice>,
        queue_family: &QueueFamily,
    ) -> Result<CommandPool, CommandPoolError> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(
                vk::CommandPoolCreateFlags::TRANSIENT
                    | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            )
            .queue_family_index(queue_family.index.unwrap());

        let handle = unsafe {
            device
                .vk_handle()
                .create_command_pool(&create_info, None)
                .map_err(|err| CommandPoolError::DeviceError(err.into()))?
        };

        Ok(CommandPool {
            buffers: Vec::new(),
            handle,
            device,
        })
    }
}

impl CommandPool {
    /// Returns [CommandBuffers][CommandBuffer] that have been allocated from this [CommandPool]
    pub fn buffers(&self) -> &[CommandBuffer] {
        &self.buffers
    }

    /// Returns a mutable list of [CommandBuffers][CommandBuffer] that have been allocated from
    /// this [CommandPool].
    pub fn buffers_mut(&mut self) -> &mut [CommandBuffer] {
        &mut self.buffers
    }
}

impl CommandPool {
    /// Allocates `count` number of [CommandBuffers][CommandBuffer] with a level of `level`
    pub fn allocate_buffers(
        &mut self,
        count: u32,
        level: CommandBufferLevel,
    ) -> Result<(), CommandPoolError> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(count)
            .command_pool(self.handle)
            .level(level.into());

        let command_buffers = unsafe {
            self.device
                .vk_handle()
                .allocate_command_buffers(&allocate_info)
                .map_err(|err| CommandPoolError::DeviceError(err.into()))?
        };
        let mut command_buffers = command_buffers
            .iter()
            .map(|&handle| CommandBuffer::new(handle, self.device.clone()))
            .collect();
        self.buffers.append(&mut command_buffers);

        Ok(())
    }

    /// Frees all the [CommandBuffers][CommandBuffer] that are currently allocated to this
    /// [CommandPool].
    pub fn free_buffers(&mut self) {
        let buffers: Vec<vk::CommandBuffer> = self
            .buffers
            .iter()
            .map(|buffer| buffer.vk_handle())
            .collect();

        unsafe {
            self.device
                .vk_handle()
                .free_command_buffers(self.handle, &buffers);
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
        };
    }
}
