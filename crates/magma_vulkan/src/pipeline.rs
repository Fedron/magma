//! This module wraps the creation of a graphics pipeline and its associated resources

use ash::vk;
use shader::ShaderStageFlags;
use std::{marker::PhantomData, rc::Rc};

use self::{
    config::PipelineConfigInfo,
    shader::{Shader, ShaderError, ShaderModule},
    vertex::Vertex,
};
use crate::{core::device::LogicalDevice, VulkanError};

pub mod config;
pub mod shader;
pub mod vertex;

/// Errors that can be thrown by the pipeline
#[derive(thiserror::Error, Debug)]
pub enum PipelineError {
    #[error("Can't create a pipeline with no shaders")]
    NoShaders,
    #[error("Failed to create pipeline layout: {0}")]
    CantCreateLayout(VulkanError),
    #[error("No render pass was set for the pipeline")]
    MissingRenderPass,
    #[error("Failed to create Vulkan pipeline: {0}")]
    CantCreatePipeline(VulkanError),
    #[error("Missing shader with shader stage: {0}")]
    MissingShader(&'static str),
    #[error("Building a shader failed: {0}")]
    ShaderError(#[from] ShaderError),
}

/// Allows you to create a graphics pipeline
pub struct PipelineBuilder<V>
where
    V: Vertex,
{
    /// Collection of shaders the pipeline will consist of
    shaders: Vec<Shader>,
    /// Render pass to use for this pipeline
    render_pass: Option<vk::RenderPass>,
    /// Fixed function configuration
    config: PipelineConfigInfo,
    phantom: PhantomData<V>,
}

impl<V> PipelineBuilder<V>
where
    V: Vertex,
{
    /// Creates a new default [PipelineBuilder]
    pub fn new() -> PipelineBuilder<V> {
        PipelineBuilder {
            shaders: Vec::new(),
            render_pass: None,
            config: PipelineConfigInfo::default(),
            phantom: PhantomData,
        }
    }

    /// Adds a [Shader] to the [PipelineBuilder]
    pub fn attach_shader(mut self, shader: Shader) -> PipelineBuilder<V> {
        self.shaders.push(shader);
        self
    }

    /// Sets the configuration of the fixed function stages in the [Pipeline]
    pub fn config(mut self, config: PipelineConfigInfo) -> PipelineBuilder<V> {
        self.config = config;
        self
    }

    /// Sets the render pass to use for the pipeline
    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> PipelineBuilder<V> {
        self.render_pass = Some(render_pass);
        self
    }

    /// Builds a [Pipeline] from the provided configuration in the [PipelineBuilder]
    ///
    /// # Errors
    /// - [PipelineError::MissingShaderStage] - If a shader with [ShaderStage::Fragment] is provided then a shader with
    /// [ShaderStage::Vertex] must also be provided.
    /// - [PipelineError::MissingRenderPass] - You need to provide a render pass for the pipeiline
    /// - [PipelineError::CantCreateLayout] and [PipelineError::CantCreatePipeline] - Failed to
    /// create required Vulkan objects, see the contained [VulkanError] for more information
    pub fn build(self, device: Rc<LogicalDevice>) -> Result<Pipeline<V>, PipelineError> {
        use std::ffi::CStr;

        if self.render_pass.is_none() {
            return Err(PipelineError::MissingRenderPass);
        }

        let mut shader_modules: Vec<ShaderModule> = Vec::new();
        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        for shader in self.shaders.iter() {
            let shader_module = shader.build(device.clone())?;

            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .module(shader_module.vk_handle())
                    // TODO: use entry point defined in shader_module
                    .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                    .stage(shader.flags.into())
                    .build(),
            );
            shader_modules.push(shader_module);
        }

        let (vertex_attribute_descriptions, vertex_binding_descriptions) = {
            let vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription> =
                V::get_attribute_descriptions()
                    .iter()
                    .map(|&description| description.into())
                    .collect();
            let vertex_binding_descriptions: Vec<vk::VertexInputBindingDescription> =
                V::get_binding_descriptions()
                    .iter()
                    .map(|&description| description.into())
                    .collect();

            (vertex_attribute_descriptions, vertex_binding_descriptions)
        };
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);

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
            _shader_modules: shader_modules,
            layout,
            handle,
            device,
            phantom: PhantomData,
        })
    }
}

/// Represents a Graphics pipeline that can be used to draw to a surface
pub struct Pipeline<V>
where
    V: Vertex,
{
    /// List of the shader modules being used by the [Pipeline]
    _shader_modules: Vec<ShaderModule>,
    /// Opaque handle to Vulkan layout used to create the pipeline
    layout: vk::PipelineLayout,
    /// Opaque handle to Vulkan Pipeline
    handle: vk::Pipeline,
    /// Logical device this pipeline belongs to
    device: Rc<LogicalDevice>,
    phantom: PhantomData<V>,
}

impl<V> Pipeline<V>
where
    V: Vertex,
{
    /// Creates a new [PipelineBuilder]
    pub fn builder() -> PipelineBuilder<V> {
        PipelineBuilder::new()
    }
}

impl<V> Pipeline<V>
where
    V: Vertex,
{
    /// Returns the handle to the Vulkan pipeline
    pub(crate) fn vk_handle(&self) -> vk::Pipeline {
        self.handle
    }
}

impl<V> Drop for Pipeline<V>
where
    V: Vertex,
{
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_pipeline_layout(self.layout, None);
            self.device.vk_handle().destroy_pipeline(self.handle, None);
        };
    }
}
