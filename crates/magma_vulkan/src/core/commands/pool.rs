use ash::vk;
use std::rc::Rc;

use super::buffer::{CommandBuffer, CommandBufferLevel};
use crate::{
    core::device::{LogicalDevice, QueueFamily},
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum CommandPoolError {
    #[error(transparent)]
    DeviceError(#[from] VulkanError),
}

pub struct CommandPool {
    buffers: Vec<CommandBuffer>,
    handle: vk::CommandPool,
    device: Rc<LogicalDevice>,
}

impl CommandPool {
    pub fn new(
        device: Rc<LogicalDevice>,
        queue_family: &QueueFamily,
    ) -> Result<CommandPool, CommandPoolError> {
        let create_info =
            vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family.index.unwrap());

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
            .map(|&handle| CommandBuffer::from(handle))
            .collect();
        self.buffers.append(&mut command_buffers);

        Ok(())
    }

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
