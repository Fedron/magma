use std::rc::Rc;

use ash::vk;
use magma_derive::{PushConstantData, Vertex};
use memoffset::offset_of;

use crate::{buffer::Buffer, device::Device, pipeline::PushConstantData};

/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputAttributeDescription.html
pub type VertexAttributeDescription = vk::VertexInputAttributeDescription;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputBindingDescription.html
pub type VertexBindingDescription = vk::VertexInputBindingDescription;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputRate.html
pub type VertexInputRate = vk::VertexInputRate;
/// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html
pub type Format = vk::Format;

/// Represents a vertex that is passed to a shader.
///
/// Allows for a struct to be passed to a [`RenderPipeline`] by providing descriptions
/// for every field in the struct.
pub trait Vertex {
    /// Returns attribute descriptions for each field in the struct.
    ///
    /// The attribute descriptions should match the layout in the vertex shader
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
    /// Returns the binding descriptions for the struct.
    ///
    /// The binding descriptions should match the layout in the vertex shader
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
}

#[derive(Vertex)]
pub struct SimpleVertex {
    #[location = 0]
    pub position: [f32; 3],
    #[location = 1]
    pub normal: [f32; 3],
    #[location = 2]
    pub color: [f32; 3],
}

#[derive(PushConstantData)]
pub struct SimplePush {
    pub offset: [f32; 2],
}

pub struct Mesh {
    /// [`Buffer`] on the GPU holding the vertices
    pub vertex_buffer: Buffer<SimpleVertex>,
    /// [`Buffer`] on the GPU holding the indices
    pub indices_buffer: Buffer<u32>,
}

impl Mesh {
    /// Creates a new [`Mesh`].
    ///
    /// Assigns the vertices and indices to new dedicated buffers on the GPU.
    pub fn new(device: Rc<Device>, vertices: &[SimpleVertex], indices: &[u32]) -> Mesh {
        if indices.len() < 3 {
            log::error!("Cannot create a model with less than 3 connected vertices");
            panic!("Failed to create model, see above");
        }

        // Copy vertices to GPU memory
        let mut staging_buffer = Buffer::<SimpleVertex>::new(
            device.clone(),
            vertices.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        staging_buffer.map(vk::WHOLE_SIZE, 0);
        staging_buffer.write(&vertices);

        let mut vertex_buffer = Buffer::<SimpleVertex>::new(
            device.clone(),
            vertices.len(),
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        vertex_buffer.copy_from(&staging_buffer);

        // Copy indices to GPU memory
        let mut staging_buffer = Buffer::<u32>::new(
            device.clone(),
            indices.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        staging_buffer.map(vk::WHOLE_SIZE, 0);
        staging_buffer.write(&indices);

        let mut indices_buffer = Buffer::<u32>::new(
            device.clone(),
            indices.len(),
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        indices_buffer.copy_from(&staging_buffer);

        Mesh {
            vertex_buffer,
            indices_buffer,
        }
    }
}
