use ash::vk;
use bitflags::bitflags;
use std::{rc::Rc, usize};

use crate::core::{
    commands::pool::CommandPool,
    device::{LogicalDevice, LogicalDeviceError, QueueFlags},
};

/// Errors that can be returned by a `Buffer"
#[derive(thiserror::Error, Debug)]
pub enum BufferError {
    #[error("Can't copy from buffer: {0}")]
    InvalidCopy(&'static str),
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
}

bitflags! {
    /// Wraps VkBufferUsageFlagBits
    pub struct BufferUsageFlags: u32 {
        /// Buffer can be used as the source of a transfer command
        const TRANSFER_SRC = 0x1;
        /// Buffer can be used as the destination of a transfer command
        const TRANSFER_DST = 0x2;
        /// Buffer can be used to create a descriptor buffer info
        const UNIFORM_BUFFER = 0x10;
        /// Buffer is able to be passed to `bind_index_buffer`
        const INDEX_BUFFER = 0x40;
        /// Buffer is able to be passed to `bind_vertex_buffer`
        const VERTEX_BUFFER = 0x80;
    }
}

impl Into<vk::BufferUsageFlags> for BufferUsageFlags {
    fn into(self) -> vk::BufferUsageFlags {
        vk::BufferUsageFlags::from_raw(self.bits())
    }
}

bitflags! {
    /// Wraps VkMemoryPropertyFlagBits
    pub struct MemoryPropertyFlags: u32 {
        /// Memory allocated is the most efficient for device access
        const DEVICE_LOCAL = 0b1;
        /// Memory allocated can be mapped for host access
        const HOST_VISIBLE = 0b10;
        /// Manually host memory flushing is not needed to make writes visible
        const HOST_COHERENT = 0b100;
    }
}

impl Into<vk::MemoryPropertyFlags> for MemoryPropertyFlags {
    fn into(self) -> vk::MemoryPropertyFlags {
        vk::MemoryPropertyFlags::from_raw(self.bits())
    }
}

/// Wraps a Vulkan buffer and device memory
pub struct Buffer<T, const CAPACITY: usize> {
    /// Mapped memory address where writes can be seen by both the device and host
    mapped: Option<*mut T>,
    /// Usage flags set on the buffer
    usage: BufferUsageFlags,
    /// Size, in bytes, of the buffer assuming the whole capacity is used up
    size: usize,

    /// Opaque object handle to Vulkan buffer
    handle: vk::Buffer,
    /// Opaque object handle to Vulkan device memory belonging to the buffer
    memory: vk::DeviceMemory,
    /// [`LogicalDevice`] the buffer and memory belong to
    device: Rc<LogicalDevice>,
}

impl<T, const CAPACITY: usize> Buffer<T, CAPACITY> {
    /// Creates a new [`Buffer`]
    pub fn new(
        device: Rc<LogicalDevice>,
        usage: BufferUsageFlags,
        memory_properties: MemoryPropertyFlags,
    ) -> Result<Buffer<T, CAPACITY>, BufferError> {
        let min_offset_alignment = if usage.contains(BufferUsageFlags::UNIFORM_BUFFER) {
            device.physical_device().properties().limits.min_uniform_buffer_offset_alignment
        } else {
            1
        };

        let instance_size = std::mem::size_of::<T>();
        let alignment_size = (instance_size + min_offset_alignment as usize - 1)
            & !(min_offset_alignment as usize - 1);
        let buffer_size = alignment_size * CAPACITY;

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
            size: buffer_size,

            handle,
            memory,
            device,
        })
    }
}

impl<T, const CAPACITY: usize> Buffer<T, CAPACITY> {
    /// Returns the number of instances of `T` that can be stored in the buffer
    pub fn capacity(&self) -> usize {
        CAPACITY 
    }

    /// Returns a corresponding descriptor buffer info if the buffer has been marked with a
    /// `BufferUsageFlag` that can be used in a descriptor set.
    pub fn descriptor(&self) -> Option<vk::DescriptorBufferInfo> {
        if self.usage.contains(BufferUsageFlags::UNIFORM_BUFFER) {
        Some(vk::DescriptorBufferInfo {
            buffer: self.handle,
            offset: 0,
            range: vk::WHOLE_SIZE,
        }) } else {
            None
        }
    }

    /// Returns a Vulkan handle to the Vulkan buffer
    pub(crate) fn vk_handle(&self) -> vk::Buffer {
        self.handle
    }
}

impl<T, const CAPACITY: usize> Buffer<T, CAPACITY> {
    /// Maps `size` amount of device memory for write access.
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

    /// Unmaps the device memory associated with the buffer meaning write operations will have no
    /// effect.
    pub fn unmap(&mut self) {
        if let Some(_) = self.mapped {
            unsafe {
                self.device.vk_handle().unmap_memory(self.memory);
            };
            self.mapped = None;
        }
    }

    /// Writes data to the buffer as long as the device memory has been mapped using
    /// [`Buffer::map`].
    ///
    /// Will unmap the memory after the write
    pub fn write(&mut self, data: &[T; CAPACITY]) {
        if let Some(mapped) = self.mapped {
            unsafe {
                mapped.copy_from_nonoverlapping(data.as_ptr(), CAPACITY);
            };
            self.unmap();
        }
    }

    /// Copies data from a buffer with the same data type and capacity to this buffer's device
    /// memory through the use of a transfer command.
    ///
    /// Requires that [`BufferUsageFlags::TRANSFER_DST`] and [`BufferUsageFlags::TRANSFER_SRC`] are
    /// set accordingly.
    pub fn copy_from(
        &mut self,
        buffer: &Buffer<T, CAPACITY>,
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
            let transfer_queue = self.device.queue(QueueFlags::GRAPHICS).ok_or(BufferError::InvalidCopy("Missing transfer queue on logical devcie"))?;

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

impl<T, const CAPACITY: usize> Drop for Buffer<T, CAPACITY> {
    fn drop(&mut self) {
        self.unmap();
        unsafe {
            self.device.vk_handle().destroy_buffer(self.handle, None);
            self.device.vk_handle().free_memory(self.memory, None);
        };
    }
}

