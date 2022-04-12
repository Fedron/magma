use ash::vk;
use bitflags::bitflags;
use std::{rc::Rc, usize};

use crate::core::{
    commands::pool::CommandPool,
    device::{LogicalDevice, LogicalDeviceError, Queue},
};

#[derive(thiserror::Error, Debug)]
pub enum BufferError {
    #[error("Can't copy from buffer: {0}")]
    InvalidCopy(&'static str),
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
}

bitflags! {
    pub struct BufferUsageFlags: u32 {
        const TRANSFER_SRC = 0x1;
        const TRANSFER_DST = 0x2;
        const UNIFORM_TEXEL = 0x4;
        const STORAGE_TEXEL = 0x8;
        const UNIFORM_BUFFER = 0x10;
        const STORAGE_BUFFER = 0x20;
        const INDEX_BUFFER = 0x40;
        const VERTEX_BUFFER = 0x80;
        const INDIRECT_BUFFER = 0x100;
    }
}

impl Into<vk::BufferUsageFlags> for BufferUsageFlags {
    fn into(self) -> vk::BufferUsageFlags {
        vk::BufferUsageFlags::from_raw(self.bits())
    }
}

bitflags! {
    pub struct MemoryPropertyFlags: u32 {
        const DEVICE_LOCAL = 0b1;
        const HOST_VISIBLE = 0b10;
        const HOST_COHERENT = 0b100;
        const HOST_CACHED = 0b1000;
        const LAZILY_ALLOCATED = 0b10000;
    }
}

impl Into<vk::MemoryPropertyFlags> for MemoryPropertyFlags {
    fn into(self) -> vk::MemoryPropertyFlags {
        vk::MemoryPropertyFlags::from_raw(self.bits())
    }
}

pub struct Buffer<T> {
    mapped: Option<*mut T>,
    usage: BufferUsageFlags,
    instance_count: u64,
    size: usize,

    handle: vk::Buffer,
    memory: vk::DeviceMemory,
    device: Rc<LogicalDevice>,
}

impl<T> Buffer<T> {
    pub fn new(
        device: Rc<LogicalDevice>,
        instance_count: usize,
        usage: BufferUsageFlags,
        memory_properties: MemoryPropertyFlags,
        min_offset_alignment: u64,
    ) -> Result<Buffer<T>, BufferError> {
        let instance_size = std::mem::size_of::<T>();
        let alignment_size = (instance_size + min_offset_alignment as usize - 1)
            & !(min_offset_alignment as usize - 1);
        let buffer_size = alignment_size * instance_count;

        let create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size as u64)
            .usage(usage.into())
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = unsafe {
            device
                .vk_handle()
                .create_buffer(&create_info, None)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        let memory_requirements =
            unsafe { device.vk_handle().get_buffer_memory_requirements(handle) };
        let memory_type =
            device.find_memory_type(memory_requirements.memory_type_bits, memory_properties)?;

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type);

        let memory = unsafe {
            device
                .vk_handle()
                .allocate_memory(&allocate_info, None)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        unsafe {
            device
                .vk_handle()
                .bind_buffer_memory(handle, memory, 0)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        Ok(Buffer {
            mapped: None,
            usage,
            instance_count: instance_count as u64,
            size: buffer_size,

            handle,
            memory,
            device,
        })
    }
}

impl<T> Buffer<T> {
    pub fn len(&self) -> usize {
        self.instance_count as usize
    }

    pub fn descriptor(&self) -> vk::DescriptorBufferInfo {
        vk::DescriptorBufferInfo {
            buffer: self.handle,
            offset: 0,
            range: vk::WHOLE_SIZE,
        }
    }

    pub(crate) fn vk_handle(&self) -> vk::Buffer {
        self.handle
    }
}

impl<T> Buffer<T> {
    pub fn map(&mut self, size: u64, offset: u64) -> Result<(), BufferError> {
        self.mapped = Some(unsafe {
            self.device
                .vk_handle()
                .map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?
                as *mut T
        });

        Ok(())
    }

    pub fn unmap(&mut self) {
        if let Some(_) = self.mapped {
            unsafe {
                self.device.vk_handle().unmap_memory(self.memory);
            };
            self.mapped = None;
        }
    }

    pub fn write(&mut self, data: &[T]) {
        if let Some(mapped) = self.mapped {
            unsafe {
                mapped.copy_from_nonoverlapping(data.as_ptr(), self.instance_count as usize);
            };
            self.unmap();
        }
    }

    pub fn copy_from(
        &mut self,
        buffer: &Buffer<T>,
        command_pool: &CommandPool,
    ) -> Result<(), BufferError> {
        if !self.usage.contains(BufferUsageFlags::TRANSFER_DST) {
            return Err(BufferError::InvalidCopy(
                "Buffer being copied to is missing TRANSFER_DST flag",
            ));
        }

        if !buffer.usage.contains(BufferUsageFlags::TRANSFER_SRC) {
            return Err(BufferError::InvalidCopy(
                "Buffer being copied from is missing TRANSFER_SRC flag",
            ));
        }

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(command_pool.vk_handle())
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            self.device
                .vk_handle()
                .allocate_command_buffers(&allocate_info)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .vk_handle()
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?;

            let copy_regions = [vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: self.size as u64,
            }];

            self.device.vk_handle().cmd_copy_buffer(
                command_buffer,
                buffer.handle,
                self.handle,
                &copy_regions,
            );

            self.device
                .vk_handle()
                .end_command_buffer(command_buffer)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?;
        };

        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            let transfer_queue = self.device.queue(Queue::Graphics).ok_or(BufferError::InvalidCopy("Missing transfer queue on logical devcie"))?;

            self.device
                .vk_handle()
                .queue_submit(
                    transfer_queue.handle,
                    &submit_infos,
                    vk::Fence::null(),
                )
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?;

            self.device
                .vk_handle()
                .queue_wait_idle(transfer_queue.handle)
                .map_err(|err| BufferError::DeviceError(LogicalDeviceError::Other(err.into())))?;

            self.device
                .vk_handle()
                .free_command_buffers(command_pool.vk_handle(), &command_buffers);
        };

        Ok(())
    }
}
