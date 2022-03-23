use std::{path::Path, rc::Rc};

use ash::vk;

use crate::{
    device::Device,
    pipeline::{Pipeline, PipelineConfigInfo},
};

pub trait Vertex {
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
}

pub trait PushConstantData {
    fn as_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            let size_in_bytes = std::mem::size_of::<Self>();
            let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
            std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
        }
    }
}

pub trait RenderSystem<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    fn new(device: Rc<Device>, render_pass: &vk::RenderPass);
    fn render(&mut self, command_buffer: vk::CommandBuffer);

    fn create_layout(device: &ash::Device) -> vk::PipelineLayout {
        let push_constant_ranges = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<P>() as u32)
            .build()];

        let layout_info =
            vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&push_constant_ranges);

        unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        }
    }

    fn create_pipeline(
        device: Rc<Device>,
        render_pass: &vk::RenderPass,
        layout: vk::PipelineLayout,
        vertex_shader_file: &Path,
        fragment_shader_file: &Path,
    ) -> Pipeline {
        let config = PipelineConfigInfo::default();
        Pipeline::new::<V>(
            device.clone(),
            config,
            render_pass,
            layout,
            vertex_shader_file,
            fragment_shader_file,
        )
    }
}
