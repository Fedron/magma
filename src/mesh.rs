use std::{path::Path, rc::Rc};

use ash::vk;
use magma_derive::Vertex;
use memoffset::offset_of;

use crate::{buffer::Buffer, device::Device};

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
pub struct OBJVertex {
    #[location = 0]
    pub position: [f32; 3],
    #[location = 1]
    pub normal: [f32; 3],
    #[location = 2]
    pub color: [f32; 3],
}

pub struct Mesh<V>
where
    V: Vertex,
{
    /// [`Buffer`] on the GPU holding the vertices
    pub vertex_buffer: Buffer<V>,
    /// [`Buffer`] on the GPU holding the indices
    pub indices_buffer: Buffer<u32>,
}

impl<V> Mesh<V>
where
    V: Vertex,
{
    /// Creates a new [`Mesh`].
    ///
    /// Assigns the vertices and indices to new dedicated buffers on the GPU.
    pub fn new(device: Rc<Device>, vertices: &[V], indices: &[u32]) -> Mesh<V> {
        if indices.len() < 3 {
            log::error!("Cannot create a model with less than 3 connected vertices");
            panic!("Failed to create model, see above");
        }

        // Copy vertices to GPU memory
        let mut staging_buffer = Buffer::<V>::new(
            device.clone(),
            vertices.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            1,
        );
        staging_buffer.map(vk::WHOLE_SIZE, 0);
        staging_buffer.write(&vertices);

        let mut vertex_buffer = Buffer::<V>::new(
            device.clone(),
            vertices.len(),
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            1,
        );
        vertex_buffer.copy_from(&staging_buffer);

        // Copy indices to GPU memory
        let mut staging_buffer = Buffer::<u32>::new(
            device.clone(),
            indices.len(),
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            1,
        );
        staging_buffer.map(vk::WHOLE_SIZE, 0);
        staging_buffer.write(&indices);

        let mut indices_buffer = Buffer::<u32>::new(
            device.clone(),
            indices.len(),
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            1,
        );
        indices_buffer.copy_from(&staging_buffer);

        Mesh {
            vertex_buffer,
            indices_buffer,
        }
    }

    /// Creates a new [`Mesh`] from an `.obj` file.
    ///
    /// If the `.obj` file contains multiple models, the first model loaded is the one that is created in `magma`.
    pub fn new_from_file(device: Rc<Device>, file: &Path) -> Mesh<OBJVertex> {
        let (models, _) =
            tobj::load_obj(file, &tobj::LoadOptions::default()).expect("Failed to load OBJ file");
        let mesh = &models
            .first()
            .expect("Failed to get first loaded models")
            .mesh;

        // Construct the vertices vector
        let mut vertices: Vec<OBJVertex> = Vec::new();
        for vertex in 0..mesh.positions.len() / 3 {
            let position = [
                mesh.positions[3 * vertex],
                mesh.positions[3 * vertex + 1],
                mesh.positions[3 * vertex + 2],
            ];

            let mut color = [1.0_f32, 1.0_f32, 1.0_f32];
            if !mesh.vertex_color.is_empty() {
                color = [
                    mesh.vertex_color[3 * vertex],
                    mesh.vertex_color[3 * vertex + 1],
                    mesh.vertex_color[3 * vertex + 2],
                ];
            }

            let mut normal = [0.0_f32, 1.0_f32, 0.0_f32];
            if !mesh.normals.is_empty() {
                normal = [
                    mesh.normals[3 * vertex],
                    mesh.normals[3 * vertex + 1],
                    mesh.normals[3 * vertex + 2],
                ];
            }

            vertices.push(OBJVertex {
                position,
                color,
                normal,
            });
        }

        Mesh::new(device.clone(), &vertices, &mesh.indices.clone())
    }
}
