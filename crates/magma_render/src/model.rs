use ash::vk;
use std::{marker::PhantomData, rc::Rc};

use crate::{
    device::{BufferUsage, Device},
    render_system::Vertex,
};

/// Represents a collection of vertices that can be drawn to the window
pub struct Model<V>
where
    V: Vertex,
{
    /// Handle to the Vulkan device used to create buffers and memory for vertex data
    device: Rc<Device>,
    /// Handle to the Vulkan buffer holding the vertex data
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBuffer.html
    vertex_buffer: vk::Buffer,
    /// Handle to the memory of the buffer holding the vertex data
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDeviceMemory.html
    vertex_buffer_memory: vk::DeviceMemory,
    /// Total number of vertices the model consists of
    vertex_count: usize,
    /// Handle to the Vulkan buffer holding the indices data
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBuffer.html
    indices_buffer: vk::Buffer,
    /// Handle to the memory of the buffer holding the indices data
    ///i
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDeviceMemory.html
    indices_buffer_memory: vk::DeviceMemory,
    phantom: PhantomData<V>,
}

impl<V> Model<V>
where
    V: Vertex,
{
    /// Creates new vertex buffers for the model on the GPU from the provided vertices
    pub fn new(device: Rc<Device>, vertices: Vec<V>, indices: Vec<u32>) -> Model<V> {
        let vertex_count = indices.len();
        if vertex_count < 3 {
            log::error!("Cannot create a model with less than 3 vertices");
            panic!("Failed to create model, see above");
        }

        let (vertex_buffer, vertex_buffer_memory) =
            device.upload_buffer_with_staging(&vertices, BufferUsage::VERTEX);

        let (indices_buffer, indices_buffer_memory) =
            device.upload_buffer_with_staging(&indices, BufferUsage::INDICES);

        Model {
            device,
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count,
            indices_buffer,
            indices_buffer_memory,
            phantom: PhantomData,
        }
    }

    /// Draws the model vertices to the command buffer
    pub fn draw(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.device.cmd_draw_indexed(
                command_buffer,
                self.vertex_count as u32,
                1,
                0,
                0,
                0,
            );
        };
    }

    /// Binds the model vertices to the command buffer
    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        let buffers = [self.vertex_buffer];
        let offsets = [0];

        unsafe {
            self.device
                .device
                .cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);

            self.device.device.cmd_bind_index_buffer(
                command_buffer,
                self.indices_buffer,
                0,
                vk::IndexType::UINT32,
            );
        };
    }
}

impl<V> Drop for Model<V>
where
    V: Vertex,
{
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_buffer(self.vertex_buffer, None);
            self.device
                .device
                .free_memory(self.vertex_buffer_memory, None);
            self.device.device.destroy_buffer(self.indices_buffer, None);
            self.device
                .device
                .free_memory(self.indices_buffer_memory, None);
        }
    }
}
