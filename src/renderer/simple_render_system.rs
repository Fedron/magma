use ash::vk;
use std::rc::Rc;

use crate::{entity::Entity, camera::Camera};

use super::{
    device::Device,
    pipeline::{Align16, Pipeline, PushConstants},
};

/// Renderers entities using the [Vulkan renderer][crate::renderer::Renderer]
///
/// The simple render system uses a graphics pipeline that uses the 'simple' shaders that can be found
/// in the 'shaders' folder
pub struct SimpleRenderSystem {
    /// Handle to logical device
    pub device: Rc<Device>,
    /// Handle to the current graphics pipeline
    pipeline: Pipeline,
}

impl SimpleRenderSystem {
    /// Creates a new simple render system
    ///
    /// The new render system will also create a new graphics pipeline for it to use that will be using the simple shaders
    pub fn new(device: Rc<Device>, render_pass: vk::RenderPass) -> SimpleRenderSystem {
        let pipeline = Pipeline::new(device.clone(), render_pass);

        SimpleRenderSystem { device, pipeline }
    }

    /// Renders the given entities using the given command buffer
    pub fn render_entities(&self, command_buffer: vk::CommandBuffer, entities: &mut Vec<Entity>, camera: &Camera) {
        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipeline,
            );

            for entity in entities.iter_mut() {
                entity.transform.rotation.y += 0.1;
                entity.transform.rotation.x += 0.05;
                entity.model().bind(command_buffer);

                let push = PushConstants {
                    transform: Align16(entity.transform_matrix() * camera.projection_matrix()),
                };

                self.device.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push.as_bytes(),
                );

                entity.model().draw(command_buffer);
            }
        };
    }
}
