use ash::vk;
use std::rc::Rc;

use self::{
    config::PipelineConfigInfo,
    shader::{Shader, ShaderStage},
};
use crate::{core::device::LogicalDevice, VulkanError};

pub mod config;
pub mod shader;

#[derive(thiserror::Error, Debug)]
pub enum PipelineError {
    #[error("Can't create a pipeline with no shaders")]
    NoShaders,
    #[error("Missing shader stage {0} to complete pipeline")]
    MissingShaderStage(ShaderStage),
    #[error("Failed to create pipeline layout")]
    CantCreateLayout(VulkanError),
    #[error("No render pass was set for the pipeline")]
    MissingRenderPass,
    #[error("Failed to create Vulkan pipeline")]
    CantCreatePipeline(VulkanError),
}

pub struct PipelineBuilder {
    shaders: Vec<Shader>,
    render_pass: Option<vk::RenderPass>,
    config: PipelineConfigInfo,
}

impl PipelineBuilder {
    pub fn new() -> PipelineBuilder {
        PipelineBuilder {
            shaders: Vec::new(),
            render_pass: None,
            config: PipelineConfigInfo::default(),
        }
    }

    pub fn add_shader(mut self, shader: Shader) -> PipelineBuilder {
        self.shaders.push(shader);
        self
    }

    pub fn config(mut self, config: PipelineConfigInfo) -> PipelineBuilder {
        self.config = config;
        self
    }

    // TODO: Set render pass function

    pub fn build(self, device: Rc<LogicalDevice>) -> Result<Pipeline, PipelineError> {
        use std::ffi::CString;

        if self
            .shaders
            .iter()
            .any(|shader| *shader.stage() == ShaderStage::Fragment)
            && !self
                .shaders
                .iter()
                .any(|shader| *shader.stage() == ShaderStage::Vertex)
        {
            return Err(PipelineError::MissingShaderStage(ShaderStage::Vertex));
        }

        if self.render_pass.is_none() {
            return Err(PipelineError::MissingRenderPass);
        }

        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            let cstring =
                unsafe { CString::from_vec_unchecked(shader.entry_point().clone().into_bytes()) };
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .module(shader.module())
                    .name(&cstring)
                    .stage(Into::<vk::ShaderStageFlags>::into(*shader.stage()))
                    .build(),
            );
        }

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[]);

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[])
            .set_layouts(&[]);

        let layout = unsafe {
            device
                .vk_handle()
                .create_pipeline_layout(&layout_create_info, None)
                .map_err(|err| PipelineError::CantCreateLayout(err.into()))?
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&self.config.input_assembly_info)
            .viewport_state(&self.config.viewport_info)
            .rasterization_state(&self.config.rasterization_info)
            .multisample_state(&self.config.multisample_info)
            .color_blend_state(&self.config.color_blend_info)
            .depth_stencil_state(&self.config.depth_stencil_info)
            .dynamic_state(&self.config.dynamic_state_info)
            .layout(layout)
            .render_pass(self.render_pass.unwrap())
            .subpass(self.config.subpass);

        let handle = unsafe {
            device
                .vk_handle()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_create_info),
                    None,
                )
                .map_err(|err| PipelineError::CantCreatePipeline(err.1.into()))?[0]
        };

        Ok(Pipeline {
            layout,
            handle,
            device,
        })
    }
}

pub struct Pipeline {
    layout: vk::PipelineLayout,
    handle: vk::Pipeline,
    device: Rc<LogicalDevice>,
}

impl Pipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_pipeline_layout(self.layout, None);
            self.device.vk_handle().destroy_pipeline(self.handle, None);
        };
    }
}
