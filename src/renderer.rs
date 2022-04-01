use ash::vk;
use std::{any::TypeId, collections::HashMap, ffi::CStr, marker::PhantomData, path::Path, rc::Rc};

use crate::{
    descriptors::{DescriptorPool, DescriptorSetLayout},
    device::Device,
    mesh::{Mesh, Vertex},
};

mod pipeline;
mod shader;

use pipeline::PipelineConfigInfo;
use shader::ShaderCompiler;
pub use shader::{NonePushConstant, Shader, UniformBuffer};

#[derive(Debug, Clone)]
pub struct UniformBufferDescription {
    pub stage: vk::ShaderStageFlags,
    pub sizes: Vec<u32>,
    pub set: u32,
    pub binding: u32,
}

pub struct RendererBuilder<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    device: Rc<Device>,
    pipeline_config: PipelineConfigInfo,
    render_pass: vk::RenderPass,
    shaders: Vec<Shader>,

    ubos: Vec<UniformBufferDescription>,
    descriptor_pool_sets: u32,

    v_phantom: PhantomData<V>,
    p_phantom: PhantomData<P>,
}

impl<V, P: 'static> RendererBuilder<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    pub fn new(device: Rc<Device>, render_pass: vk::RenderPass) -> RendererBuilder<V, P> {
        RendererBuilder {
            device,
            pipeline_config: PipelineConfigInfo::default(),
            render_pass,
            shaders: Vec::new(),

            ubos: Vec::new(),
            descriptor_pool_sets: 100,

            v_phantom: PhantomData,
            p_phantom: PhantomData,
        }
    }

    pub fn pipeline_config(mut self, config: PipelineConfigInfo) -> RendererBuilder<V, P> {
        self.pipeline_config = config;
        self
    }

    pub fn render_pass(mut self, render_pass: vk::RenderPass) -> RendererBuilder<V, P> {
        self.render_pass = render_pass;
        self
    }

    pub fn max_descriptor_pool_sets(mut self, max_sets: u32) -> RendererBuilder<V, P> {
        self.descriptor_pool_sets = max_sets;
        self
    }

    pub fn add_shader(mut self, shader: Shader) -> RendererBuilder<V, P> {
        self.shaders.push(shader);
        self
    }

    pub fn add_ubo<U: 'static>(mut self, set: u32, binding: u32) -> RendererBuilder<V, P>
    where
        U: UniformBuffer,
    {
        self.ubos.push(UniformBufferDescription {
            stage: U::stage(),
            sizes: U::sizes(),
            set,
            binding,
        });
        self
    }

    pub fn build(mut self) -> Renderer<V, P> {
        let mut shader_modules: Vec<vk::ShaderModule> = Vec::with_capacity(self.shaders.len());
        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            Vec::with_capacity(self.shaders.len());

        for shader in self.shaders.iter() {
            let compiler = ShaderCompiler::new(shader.clone());
            if shader.stage == Shader::VERTEX {
                compiler.check_vertex_attributes::<V>();
            }

            if P::stage().contains(shader.stage) {
                compiler.check_push_constant::<P>();
            }

            compiler.check_ubos(
                self.ubos
                    .iter()
                    .filter(|&ubo| ubo.stage.contains(shader.stage))
                    .map(|ubo| ubo.clone())
                    .collect(),
            );

            let module = RendererBuilder::<V, P>::create_shader_module(
                self.device.vk(),
                Path::new(&shader.file),
            );
            shader_modules.push(module);

            let name =
                unsafe { CStr::from_bytes_with_nul_unchecked(shader.entry_point.as_bytes()) };
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::builder()
                    .module(module)
                    .name(name)
                    .stage(shader.stage)
                    .build(),
            );
        }

        // Create the graphics pipeline
        let attribute_descriptions = V::get_attribute_descriptions();
        let binding_descriptions = V::get_binding_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attribute_descriptions)
            .vertex_binding_descriptions(&binding_descriptions);

        let mut push_constant_ranges: Vec<vk::PushConstantRange> = Vec::new();
        if TypeId::of::<P>() != TypeId::of::<NonePushConstant>() {
            push_constant_ranges.push(
                vk::PushConstantRange::builder()
                    .stage_flags(P::stage())
                    .offset(0)
                    .size(std::mem::size_of::<P>() as u32)
                    .build(),
            );
        };

        self.ubos.sort_by(|a, b| a.set.cmp(&b.set));
        let mut descriptor_set_layouts: HashMap<u32, Vec<vk::DescriptorSetLayoutBinding>> =
            HashMap::new();
        for ubo in self.ubos.iter() {
            if !descriptor_set_layouts.contains_key(&ubo.set) {
                descriptor_set_layouts.insert(ubo.set, Vec::new());
            }

            let descriptor_set_layout = descriptor_set_layouts.get_mut(&ubo.set).unwrap();
            descriptor_set_layout.push(
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(ubo.binding)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(ubo.stage)
                    .build(),
            );
        }

        let mut layouts: Vec<vk::DescriptorSetLayout> = Vec::new();
        for (_, bindings) in descriptor_set_layouts.iter() {
            let layout = DescriptorSetLayout::new(self.device.clone(), &bindings);
            layouts.push(layout.layout);
        }

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&layouts);
        let pipeline_layout = unsafe {
            self.device
                .vk()
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&self.pipeline_config.input_assembly_info)
            .viewport_state(&self.pipeline_config.viewport_info)
            .rasterization_state(&self.pipeline_config.rasterization_info)
            .multisample_state(&self.pipeline_config.multisample_info)
            .color_blend_state(&self.pipeline_config.color_blend_info)
            .depth_stencil_state(&self.pipeline_config.depth_stencil_info)
            .dynamic_state(&self.pipeline_config.dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(self.pipeline_config.subpass);

        let pipeline = unsafe {
            self.device
                .vk()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|e| log::error!("Unable to create graphics pipeline: {:?}", e))
                .unwrap()[0]
        };

        for &module in shader_modules.iter() {
            unsafe {
                self.device.vk().destroy_shader_module(module, None);
            };
        }

        // Create descriptor pool and buffers for any ubos
        let descriptor_pool: Option<DescriptorPool> = if self.ubos.len() == 0 {
            None
        } else {
            Some(
                DescriptorPool::builder(self.device.clone())
                    .max_sets(self.descriptor_pool_sets)
                    .add_pool_size(
                        vk::DescriptorType::UNIFORM_BUFFER,
                        self.descriptor_pool_sets as u32,
                    )
                    .build(),
            )
        };

        Renderer {
            device: self.device,
            pipeline,
            pipeline_layout,

            descriptor_pool,
            descriptor_layouts: layouts,

            meshes: Vec::new(),
            is_none_push_constant: TypeId::of::<P>() == TypeId::of::<NonePushConstant>(),
            push_constant: None,
        }
    }
}

impl<V, P> RendererBuilder<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    /// Creates a new Vulkan shader module from the shader file at the Path provided.
    ///
    /// Will panic if a file at the [`Path`] could not be found. If the file is found
    /// but not a valid SPIR-V the function will panic.
    ///
    /// The `.spv` extension is automatically added to the end of the [`Path`].
    fn create_shader_module(device: &ash::Device, shader_path: &Path) -> vk::ShaderModule {
        let mut shader_path = shader_path.to_path_buf();
        shader_path.set_extension(format!(
            "{}.spv",
            shader_path.extension().unwrap().to_str().unwrap()
        ));
        let shader_code = ash::util::read_spv(
            &mut std::fs::File::open(shader_path).expect("Failed to open file"),
        )
        .expect("Failed to read spv");

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&shader_code);

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }
}

pub trait DrawRenderer {
    fn draw(&self, command_buffer: vk::CommandBuffer);
}

pub struct Renderer<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    device: Rc<Device>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    descriptor_pool: Option<DescriptorPool>,
    descriptor_layouts: Vec<vk::DescriptorSetLayout>,

    meshes: Vec<Mesh<V>>,
    is_none_push_constant: bool,
    push_constant: Option<P>,
}

impl<V, P: 'static> Renderer<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    pub fn builder(device: Rc<Device>, render_pass: vk::RenderPass) -> RendererBuilder<V, P> {
        RendererBuilder::new(device, render_pass)
    }

    pub fn add_mesh(&mut self, mesh: Mesh<V>) {
        self.meshes.push(mesh);
    }

    pub fn set_push_constant(&mut self, push_constant: P) {
        self.push_constant = Some(push_constant);
    }
}

impl<V, P> DrawRenderer for Renderer<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    fn draw(&self, command_buffer: vk::CommandBuffer) {
        if !self.is_none_push_constant && self.push_constant.is_none() {
            log::warn!("Push constant not assigned in renderer, aborting draw");
            return;
        }

        unsafe {
            self.device.vk().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            if !self.is_none_push_constant {
                let push_constant = self.push_constant.as_ref().unwrap();
                self.device.vk().cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    P::stage(),
                    0,
                    push_constant.as_bytes(),
                );
            }
        };

        for mesh in self.meshes.iter() {
            let buffers = [mesh.vertex_buffer.vk()];
            let offsets = [0];

            unsafe {
                self.device
                    .vk()
                    .cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);
                self.device.vk().cmd_bind_index_buffer(
                    command_buffer,
                    mesh.indices_buffer.vk(),
                    0,
                    vk::IndexType::UINT32,
                );

                self.device.vk().cmd_draw_indexed(
                    command_buffer,
                    mesh.indices_buffer.len() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            };
        }
    }
}

impl<V, P> Drop for Renderer<V, P>
where
    V: Vertex,
    P: UniformBuffer,
{
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_pipeline(self.pipeline, None);
            self.device
                .vk()
                .destroy_pipeline_layout(self.pipeline_layout, None);

            for &layout in self.descriptor_layouts.iter() {
                self.device.vk().destroy_descriptor_set_layout(layout, None);
            }
        };
    }
}
