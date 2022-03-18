use ash::vk;
use std::rc::Rc;

use crate::entity::Entity;

use super::{device::Device, pipeline::{Pipeline, PushConstants, Align16}};

pub struct SimpleRenderSystem {
    /// Handle to logical device
    pub device: Rc<Device>,
    /// Handle to the current graphics pipeline
    pipeline: Pipeline,
}

impl SimpleRenderSystem {
    pub fn new(device: Rc<Device>, render_pass: vk::RenderPass) -> SimpleRenderSystem {
        let pipeline = Pipeline::new(device.clone(), render_pass);

        SimpleRenderSystem {
            device,
            pipeline,
        }
    }

    pub fn render_entities(
        &self,
        command_buffer: vk::CommandBuffer,
        entities: &mut Vec<Entity>,
    ) {
        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.graphics_pipeline,
            );

            for entity in entities.iter_mut() {
                entity.model().bind(command_buffer);
                entity.transform.rotation += 0.1;

                let push = PushConstants {
                    transform: Align16(entity.transform_matrix()),
                    translation: Align16(entity.transform.position),
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
