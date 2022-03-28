use ash::vk;
use magma_derive::Vertex;
use memoffset::offset_of;
use std::{marker::PhantomData, path::Path, rc::Rc};

use crate::{
    device::{BufferUsage, Device},
    renderer::{
        Format, PushConstantData, Vertex, VertexAttributeDescription, VertexBindingDescription,
        VertexInputRate,
    },
};

#[repr(C)]
#[derive(Vertex)]
pub struct OBJVertex {
    #[location = 0]
    pub position: [f32; 3],
    #[location = 1]
    pub color: [f32; 3],
    #[location = 2]
    pub normal: [f32; 3],
    #[location = 3]
    pub uv: [f32; 2],
}

/// Represents a collection of [`Vertex`] that can be drawn using a [`RenderPipeline`]
/// of the same [`Vertex`] and [`PushConstantData`] types.
pub struct Model<P, V>
where
    P: PushConstantData,
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
    /// Total number of vertices the [`Model`] consists of
    vertex_count: usize,
    /// Handle to the Vulkan buffer holding the indices data
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBuffer.html
    indices_buffer: vk::Buffer,
    /// Handle to the memory of the buffer holding the indices data
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDeviceMemory.html
    indices_buffer_memory: vk::DeviceMemory,
    /// Contains the [`PushConstantData`] to push when the [`Model`] is drawn.
    ///
    /// Should be set before the [`Model::draw`] is called.
    push_constants: Option<P>,
    vertex_phantom: PhantomData<V>,
}

impl<P, V> Model<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    /// Creates a new [`Model`].
    ///
    /// Assigns the vertices and indices to new dedicated buffers on the GPU.
    /// The [`PushConstantData`] is set to `None` and should be set before a call to
    /// [`Model::draw`].
    pub fn new(device: Rc<Device>, vertices: Vec<V>, indices: Vec<u32>) -> Model<P, V> {
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
            push_constants: None,
            vertex_phantom: PhantomData,
        }
    }

    /// Creates a new [`Model`] with the [`PushConstantData`].
    ///
    /// Assigns the vertices and indices to new dedicated buffers on the GPU.
    pub fn new_with_push(
        device: Rc<Device>,
        vertices: Vec<V>,
        indices: Vec<u32>,
        push_constants: P,
    ) -> Model<P, V> {
        let mut model = Model::new(device.clone(), vertices, indices);
        model.set_push_constants(push_constants);
        model
    }

    /// Creates a new [`Model`] from an `.obj` file.
    ///
    /// If the `.obj` file contains multiple models, the first model loaded is the one that is created in `magma`.
    pub fn new_from_file(device: Rc<Device>, file: &Path) -> Model<P, OBJVertex> {
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

            // TODO: Read texture coords
            vertices.push(OBJVertex {
                position,
                color,
                normal,
                uv: [0.0, 0.0],
            });
        }

        Model::<P, OBJVertex>::new(device.clone(), vertices, mesh.indices.clone())
    }

    /// Sets the [`PushConstantData`] on the [`Model`]
    pub fn set_push_constants(&mut self, push_constants: P) {
        self.push_constants = Some(push_constants);
    }

    /// Draws the [`Model`].
    ///
    /// If the [`PushConstantData`] wasn't set prior to this function being called, the
    /// [`Model`] won't be drawn.
    pub fn draw(&self, command_buffer: vk::CommandBuffer, layout: vk::PipelineLayout) {
        if self.push_constants.is_none() {
            log::warn!("You haven't set push constant data so the model won't be drawn");
            return;
        }

        // Bind
        let buffers = [self.vertex_buffer];
        let offsets = [0];

        unsafe {
            // Bind
            self.device
                .device
                .cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);

            self.device.device.cmd_bind_index_buffer(
                command_buffer,
                self.indices_buffer,
                0,
                vk::IndexType::UINT32,
            );

            if let Some(push_constants) = &self.push_constants {
                self.device.device.cmd_push_constants(
                    command_buffer,
                    layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push_constants.as_bytes(),
                )
            }
            // Draw
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
}

impl<P, V> Drop for Model<P, V>
where
    P: PushConstantData,
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
