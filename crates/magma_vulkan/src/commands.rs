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

    buffers: Vec<vk::CommandBuffer>,
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

        self.buffers.append(&mut unsafe {
            self.device
                .vk_handle()
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffers")
        });
    }

    pub fn free_buffers(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .free_command_buffers(self.handle, &self.buffers);
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
