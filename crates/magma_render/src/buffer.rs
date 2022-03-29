use ash::vk;
use std::rc::Rc;

use crate::device::Device;

/// Represents a GPU memory buffer
pub struct Buffer<T> {
    /// [`Device`] to which this [`Buffer`] belongs
    device: Rc<Device>,
    /// Handle to the underlying buffer object
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBuffer.html
    buffer: vk::Buffer,
    /// Handle to the GPU memory for the buffer
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDeviceMemory.html
    memory: vk::DeviceMemory,
    /// Pointer to the beginning of mapped buffer memory
    mapped: Option<*mut T>,
    /// Usage flags the [`Buffer`] was created with
    usage: vk::BufferUsageFlags,
    /// Number of `T` in the [`Buffer`]
    instance_count: u64,
    /// Size of the [`Buffer`] in bytes
    size: usize,
}

impl<T> Buffer<T> {
    /// Creates a new [`Buffer`].
    ///
    /// Creates a new [`ash::vk::Buffer`] and [`ash::vk::DeviceMemory`] on the `device` passed in.
    /// The usage and memory properties of a [`Buffer`] cannot be changed after creation.
    pub fn new(
        device: Rc<Device>,
        instance_count: usize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Buffer<T> {
        let buffer_size = std::mem::size_of::<T>() * instance_count;

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size as u64)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device
                .device
                .create_buffer(&buffer_info, None)
                .expect("Failed to create buffer")
        };

        let memory_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
        let memory_type =
            device.find_memory_type(memory_requirements.memory_type_bits, memory_properties);

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type);

        let buffer_memory = unsafe {
            device
                .device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate buffer memory")
        };

        unsafe {
            device
                .device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Failed to bind buffer memory");
        };

        Buffer {
            device: device.clone(),
            buffer,
            memory: buffer_memory,
            mapped: None,
            usage,
            instance_count: instance_count as u64,
            size: buffer_size,
        }
    }

    /// Gets the handle to the [`Vulkan buffer object`][ash::vk::Buffer]
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    /// Maps a region of the [`Buffer`]s memory so that it can be written to.
    ///
    /// If you want to map the whole buffer, use vk::WHOLE_SIZE or u64::MAX.
    pub fn map(&mut self, size: vk::DeviceSize, offset: vk::DeviceSize) {
        self.mapped = Some(unsafe {
            self.device
                .device
                .map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut T
        });
    }

    /// If the [`Buffer`] memory was mapped previously then it is unmapped.
    pub fn unmap(&mut self) {
        if let Some(_) = self.mapped {
            unsafe {
                self.device.device.unmap_memory(self.memory);
            };
            self.mapped = None;
        }
    }

    /// Writes data to a mapped region of the [`Buffer`]s memory.
    ///
    /// It is assumed that `data` has a length of `instance_count`, which is set when you created
    /// the [`Buffer`]. Will only write `data` if [`Buffer::map`] was called prior to [`Buffer::write`].
    pub fn write(&mut self, data: &[T]) {
        if let Some(mapped) = self.mapped {
            unsafe {
                mapped.copy_from_nonoverlapping(data.as_ptr(), self.instance_count as usize);
            };
            self.unmap();
        }
    }

    /// Copies data from `buffer` to this [`Buffer`].
    /// 
    /// Requires that `buffer` has a TRANSFER_SRC usage flag, and that this [`Buffer`] has a
    /// TRANSFER_DST usage flag. Will panic if this requirement is not met.
    pub fn copy_from(&mut self, buffer: &Buffer<T>) {
        if !self.usage.contains(vk::BufferUsageFlags::TRANSFER_DST) {
            panic!("Can't copy into buffer as it isn't flagged as TRANSFER_DST")
        }

        if !buffer.usage.contains(vk::BufferUsageFlags::TRANSFER_SRC) {
            panic!(
                "Can't copy into buffer as buffer being copied from isn't flagged as TRANSFER_SRC"
            )
        }

        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(1)
            .command_pool(self.device.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            self.device
                .device
                .allocate_command_buffers(&allocate_info)
                .expect("Failed to allocate command buffer")
        };
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin command buffer");

            let copy_regions = [vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: self.size as u64,
            }];

            self.device.device.cmd_copy_buffer(
                command_buffer,
                buffer.buffer,
                self.buffer,
                &copy_regions,
            );

            self.device
                .device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer");
        };

        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            self.device
                .device
                .queue_submit(self.device.graphics_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to submit queue");

            self.device
                .device
                .queue_wait_idle(self.device.graphics_queue)
                .expect("Failed to wait for submit queue to finish");

            self.device
                .device
                .free_command_buffers(self.device.command_pool, &command_buffers);
        };
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        self.unmap();
        unsafe {
            self.device.device.destroy_buffer(self.buffer, None);
            self.device.device.free_memory(self.memory, None);
        };
    }
}
