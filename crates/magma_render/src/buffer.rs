use ash::vk;
use std::rc::Rc;

use crate::device::Device;

/// Wraps [`BufferUsageFlags`][ash::vk::BufferUsageFlags] with the specific flags that [`Device`] supports
#[derive(PartialEq)]
pub struct BufferUsage(pub vk::BufferUsageFlags);
impl BufferUsage {
    pub const VERTEX: BufferUsage = BufferUsage(vk::BufferUsageFlags::VERTEX_BUFFER);
    pub const INDICES: BufferUsage = BufferUsage(vk::BufferUsageFlags::INDEX_BUFFER);
}

pub struct Buffer<T> {
    device: Rc<Device>,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped: Option<*mut T>,
    instance_count: u64,
}

impl<T> Buffer<T> {
    pub fn new(
        device: Rc<Device>,
        instance_size: vk::DeviceSize,
        instance_count: u64,
        usage_flags: BufferUsage,
        memory_properties: vk::MemoryPropertyFlags,
        min_offset_alignment: vk::DeviceSize,
    ) -> Buffer<T> {
        let alignment_size = Buffer::<T>::get_alignment(instance_size, min_offset_alignment);
        let buffer_size = alignment_size * instance_count;

        let (buffer, memory) = device.create_buffer(buffer_size, usage_flags.0, memory_properties);

        Buffer {
            device: device.clone(),
            buffer,
            memory,
            mapped: None,
            instance_count,
        }
    }

    pub fn get_alignment(
        instance_size: vk::DeviceSize,
        min_offset_alignment: vk::DeviceSize,
    ) -> vk::DeviceSize {
        if min_offset_alignment > 0 {
            (instance_size + min_offset_alignment - 1) & !(min_offset_alignment - 1)
        } else {
            instance_size
        }
    }

    pub fn map(&mut self, size: vk::DeviceSize, offset: vk::DeviceSize) {
        self.mapped = Some(unsafe {
            self.device
                .device
                .map_memory(self.memory, offset, size, vk::MemoryMapFlags::empty())
                .expect("Failed to map memory") as *mut T
        });
    }

    pub fn unmap(&mut self) {
        if let Some(_) = self.mapped {
            unsafe {
                self.device.device.unmap_memory(self.memory);
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
