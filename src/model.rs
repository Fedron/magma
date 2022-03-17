use std::{fmt::Debug, rc::Rc};

use ash::vk;
use memoffset::offset_of;

use crate::vulkan::device::{BufferUsage, Device};

/// Represents a singe vertex with a 2D position and colour
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    /// Gets the binding descriptions for the vertex buffer
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    /// Gets the attribute descriptions for the vertex buffer
    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, position) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ]
    }
}

/// Represents a collection of vertices that can be drawn to the window
pub struct Model {
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
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDeviceMemory.html
    indices_buffer_memory: vk::DeviceMemory,
}

impl Model {
    /// Creates new vertex buffers for the model on the GPU from the provided vertices
    pub fn new(device: Rc<Device>, vertices: Vec<Vertex>, indices: Vec<u32>) -> Model {
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
        }
    }

    /// Draws the model vertices to the command buffer
    pub fn draw(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .device
                .cmd_draw_indexed(command_buffer, self.vertex_count as u32, 1, 0, 0, 0);
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

impl Drop for Model {
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
