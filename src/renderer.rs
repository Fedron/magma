use ash::vk;
use std::{ffi::CStr, marker::PhantomData, path::Path, rc::Rc};

use crate::{
    device::Device,
    mesh::{Mesh, Vertex},
};

mod pipeline;
use pipeline::PipelineConfigInfo;

pub struct Shader {
    pub file: &'static str,
    pub entry_point: &'static str,
    pub stage: vk::ShaderStageFlags,
}

impl Shader {
    pub const VERTEX: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
    pub const FRAGMENT: vk::ShaderStageFlags = vk::ShaderStageFlags::FRAGMENT;
}

pub struct RendererBuilder<V>
where
    V: Vertex,
{
    device: Rc<Device>,
    pipeline_config: PipelineConfigInfo,
    render_pass: vk::RenderPass,
    shaders: Vec<Shader>,
    phantom: PhantomData<V>,
}

impl<V> RendererBuilder<V>
where
    V: Vertex,
{
    pub fn new(device: Rc<Device>, render_pass: vk::RenderPass) -> RendererBuilder<V> {
        RendererBuilder {
            device,
            pipeline_config: PipelineConfigInfo::default(),
            render_pass: render_pass,
            shaders: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn pipeline_config(mut self, config: PipelineConfigInfo) -> RendererBuilder<V> {
        self.pipeline_config = config;
        self
    }

    pub fn add_shader(mut self, shader: Shader) -> RendererBuilder<V> {
        self.shaders.push(shader);
        self
    }

    pub fn build(self) -> Renderer<V> {
        let mut shader_modules: Vec<vk::ShaderModule> = Vec::with_capacity(self.shaders.len());
        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            Vec::with_capacity(self.shaders.len());

        for shader in self.shaders.iter() {
            let module = RendererBuilder::<V>::create_shader_module(
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

        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[])
            .set_layouts(&[]);
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

        Renderer {
            device: self.device,
            pipeline,
            pipeline_layout,
            meshes: Vec::new(),
        }
    }
}

impl<V> RendererBuilder<V>
where
    V: Vertex,
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

pub struct Renderer<V>
where
    V: Vertex,
{
    device: Rc<Device>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    meshes: Vec<Mesh<V>>,
}

impl<V> Renderer<V>
where
    V: Vertex,
{
    pub fn builder(device: Rc<Device>, render_pass: vk::RenderPass) -> RendererBuilder<V> {
        RendererBuilder::new(device, render_pass)
    }

    pub fn add_mesh(&mut self, mesh: Mesh<V>) {
        self.meshes.push(mesh);
    }
}

impl<V> DrawRenderer for Renderer<V>
where
    V: Vertex,
{
    fn draw(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.vk().cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
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

impl<V> Drop for Renderer<V>
where
    V: Vertex,
{
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_pipeline(self.pipeline, None);
            self.device
                .vk()
                .destroy_pipeline_layout(self.pipeline_layout, None);
        };
    }
}
