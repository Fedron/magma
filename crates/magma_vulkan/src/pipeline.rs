//! This module wraps the creation of a graphics pipeline and its associated resources

use ash::vk;
use shader::ShaderStageFlags;
use std::{any::TypeId, marker::PhantomData, rc::Rc};

use self::{
    config::PipelineConfigInfo,
    shader::{Shader, ShaderError, ShaderModule},
    ubo::{EmptyPushConstant, UniformBuffer},
    vertex::{EmptyVertex, Vertex},
};
use crate::{
    core::{commands::buffer::CommandBuffer, device::LogicalDevice},
    descriptors::DescriptorSetLayout,
    VulkanError,
};

pub mod config;
pub mod shader;
pub mod ubo;
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
    MissingShader(String),
    #[error("Building a shader failed: {0}")]
    ShaderError(#[from] ShaderError),
}

/// Allows you to create a graphics pipeline
pub struct PipelineBuilder<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// Collection of shaders the pipeline will consist of
    shaders: Vec<Shader>,
    /// Render pass to use for this pipeline
    render_pass: Option<vk::RenderPass>,
    /// Fixed function configuration
    config: PipelineConfigInfo,
    v_phantom: PhantomData<V>,
    p_phantom: PhantomData<P>,
}

impl<V: 'static, P: 'static> PipelineBuilder<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// Creates a new default [PipelineBuilder]
    pub fn new() -> PipelineBuilder<V, P> {
        PipelineBuilder {
            shaders: Vec::new(),
            render_pass: None,
            config: PipelineConfigInfo::default(),
            v_phantom: PhantomData,
            p_phantom: PhantomData,
        }
    }

    /// Adds a [Shader] to the [PipelineBuilder]
    pub fn attach_shader(mut self, shader: Shader) -> PipelineBuilder<V, P> {
        self.shaders.push(shader);
        self
    }

    /// Sets the configuration of the fixed function stages in the [Pipeline]
    pub fn config(mut self, config: PipelineConfigInfo) -> PipelineBuilder<V, P> {
        self.config = config;
        self
    }

    /// Sets the render pass to use for the pipeline
    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> PipelineBuilder<V, P> {
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
    pub fn build(self, device: Rc<LogicalDevice>) -> Result<Pipeline<V, P>, PipelineError> {
        if self.render_pass.is_none() {
            return Err(PipelineError::MissingRenderPass);
        }

        if TypeId::of::<V>() != TypeId::of::<EmptyVertex>() {
            let vertex_shader = self
                .shaders
                .iter()
                .find(|&shader| shader.flags.contains(ShaderStageFlags::VERTEX));

            if vertex_shader.is_none() {
                return Err(PipelineError::MissingShader("Pipeline has a non-empty vertex type, yet no vertex shader was attached to the pipeline".to_string()));
            } else {
                vertex_shader.unwrap().check_vertex_input::<V>()?;
            }
        }

        if TypeId::of::<P>() != TypeId::of::<EmptyPushConstant>() {
            let shaders: Vec<&Shader> = self
                .shaders
                .iter()
                .filter(|&shader| shader.flags.intersects(P::stage()))
                .collect();

            // Remove each flag from each shader we have to see if we are missing any shader stages
            // the push constant requires
            let mut required_shaders = P::stage();
            for &shader in shaders.iter() {
                required_shaders.remove(shader.flags);
            }
            if !required_shaders.is_empty() {
                return Err(
                    PipelineError::MissingShader(
                        format!(
                            "Pipeline has a non-empty push constant type that requires {:#?} shaders, but shaders with flags {:#?} were not attached to the pipeline",
                            P::stage(),
                            required_shaders
                    )));
            }

            for &shader in shaders.iter() {
                shader.check_push_constant::<P>()?;
            }
        }

        let mut shader_modules: Vec<ShaderModule> = Vec::new();
        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = Vec::new();
        let mut set_layouts: Vec<vk::DescriptorSetLayout> = Vec::new();
        for shader in self.shaders.iter() {
            let shader_module = shader.build(device.clone())?;
            set_layouts.append(&mut shader.get_descriptor_set_layouts()?);

            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .module(shader_module.vk_handle())
                    .name(&shader.entry_point)
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

        let mut push_constant_ranges: Vec<vk::PushConstantRange> = Vec::new();
        if TypeId::of::<P>() != TypeId::of::<EmptyPushConstant>() {
            push_constant_ranges.push(
                vk::PushConstantRange::builder()
                    .stage_flags(P::stage().into())
                    .offset(0)
                    .size(std::mem::size_of::<P>() as u32)
                    .build(),
            );
        }

        let layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&set_layouts);

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
            v_phantom: PhantomData,
            p_phantom: PhantomData,
        })
    }
}

/// Represents a Graphics pipeline that can be used to draw to a surface
pub struct Pipeline<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// List of the shader modules being used by the [Pipeline]
    _shader_modules: Vec<ShaderModule>,
    /// Opaque handle to Vulkan layout used to create the pipeline
    layout: vk::PipelineLayout,
    /// Opaque handle to Vulkan Pipeline
    handle: vk::Pipeline,
    /// Logical device this pipeline belongs to
    device: Rc<LogicalDevice>,
    v_phantom: PhantomData<V>,
    p_phantom: PhantomData<P>,
}

impl<V: 'static, P: 'static> Pipeline<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// Creates a new [PipelineBuilder]
    pub fn builder() -> PipelineBuilder<V, P> {
        PipelineBuilder::new()
    }
}

impl<V, P> Pipeline<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// Returns the handle to the Vulkan pipeline
    pub(crate) fn vk_handle(&self) -> vk::Pipeline {
        self.handle
    }
}

impl<V, P> Pipeline<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    pub fn set_push_constant(&self, command_buffer: &CommandBuffer, data: P) {
        unsafe {
            self.device.vk_handle().cmd_push_constants(
                command_buffer.vk_handle(),
                self.layout,
                P::stage().into(),
                0,
                data.as_bytes(),
            );
        };
    }

    pub fn set_descriptor_sets(&self, command_buffer: &CommandBuffer, sets: &[vk::DescriptorSet]) {
        unsafe {
            self.device.vk_handle().cmd_bind_descriptor_sets(
                command_buffer.vk_handle(),
                vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                sets,
                &[],
            );
        };
    }
}

impl<V, P> Drop for Pipeline<V, P>
where
    V: Vertex,
    P: UniformBuffer,
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
