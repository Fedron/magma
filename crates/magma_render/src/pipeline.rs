use ash::vk;
use std::{cell::RefCell, ffi::CString, path::Path, rc::Rc};

use crate::{
    device::Device,
    model::Model,
    renderer::{PushConstantData, Vertex},
};

/// Represents the possible shader stages, wraps [`ash::vk::ShaderStageFlags`]
pub struct ShaderStageFlag(vk::ShaderStageFlags);
impl ShaderStageFlag {
    pub const VERTEX: ShaderStageFlag = ShaderStageFlag(vk::ShaderStageFlags::VERTEX);
    pub const FRAGMENT: ShaderStageFlag = ShaderStageFlag(vk::ShaderStageFlags::FRAGMENT);
}

/// Wraps various Vulkan create infos needed to create a [`Pipeline`]
pub struct PipelineConfigInfo {
    viewport_info: vk::PipelineViewportStateCreateInfo,
    input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    multisample_info: vk::PipelineMultisampleStateCreateInfo,
    _color_blend_attachment: Rc<vk::PipelineColorBlendAttachmentState>,
    color_blend_info: vk::PipelineColorBlendStateCreateInfo,
    depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    _dynamic_state_enables: Vec<vk::DynamicState>,
    dynamic_state_info: vk::PipelineDynamicStateCreateInfo,
    subpass: u32,
}

impl Default for PipelineConfigInfo {
    fn default() -> PipelineConfigInfo {
        let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL)
            .rasterizer_discard_enable(false)
            .depth_bias_clamp(0.0)
            .depth_bias_constant_factor(0.0)
            .depth_bias_enable(false)
            .depth_bias_slope_factor(0.0)
            .build();

        let multisample_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .sample_mask(&[])
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false)
            .build();

        let color_blend_attachment = Rc::new(
            vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ZERO)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .build(),
        );

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0)
            .write_mask(0)
            .reference(0)
            .build();

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
            .build();

        let dynamic_state_enables = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state_enables)
            .build();

        PipelineConfigInfo {
            viewport_info,
            input_assembly_info,
            rasterization_info,
            multisample_info,
            _color_blend_attachment: color_blend_attachment,
            color_blend_info,
            depth_stencil_info,
            _dynamic_state_enables: dynamic_state_enables,
            dynamic_state_info,
            subpass: 0,
        }
    }
}

/// Represents a [`Pipeline`] that can draw to a surface
pub trait RenderPipeline {
    /// Draws all the [`Model`]s in a [`Pipeline`] to the command buffer
    fn draw(&self, command_buffer: vk::CommandBuffer);
}

pub struct Pipeline<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    /// Handle to the [`Device`] being used to draw
    device: Rc<Device>,
    /// Handle to the Vulkan pipeline that can be bound to draw graphics
    pub graphics_pipeline: vk::Pipeline,
    /// Handle to the layout of the pipeline
    pub layout: vk::PipelineLayout,
    /// List of all the [`Model`]s that will be drawn by this [`Pipeline`]
    models: Vec<Rc<RefCell<Model<P, V>>>>,
}

impl<P, V> Pipeline<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    /// Creates a new [`Pipeline`].
    ///
    /// The [`Pipeline`] will only be able to draw [`Model`]s with [`Vertex`] and [`PushConstantData`]
    /// of the same type.
    ///
    /// The [`PushConstantData`] will be bound to the shader stages specified by `push_bind_flag`.
    pub fn new(
        device: Rc<Device>,
        config: PipelineConfigInfo,
        render_pass: &vk::RenderPass,
        vertex_shader_file: &Path,
        fragment_shader_file: &Path,
        push_bind_flag: ShaderStageFlag,
    ) -> Pipeline<P, V> {
        let vertex_shader_module =
            Pipeline::<P, V>::create_shader_module(&device.as_ref().device, vertex_shader_file);
        let fragment_shader_module =
            Pipeline::<P, V>::create_shader_module(&device.as_ref().device, fragment_shader_file);

        let entry_point = CString::new("main").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vertex_shader_module)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(fragment_shader_module)
                .name(&entry_point)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        // Create the graphics pipeline
        let attribute_descriptions = V::get_attribute_descriptions();
        let binding_descriptions = V::get_binding_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attribute_descriptions)
            .vertex_binding_descriptions(&binding_descriptions);

        let push_constant_size = std::mem::size_of::<P>() as u32;
        let push_constant_ranges = if push_constant_size == 0 {
            vec![]
        } else {
            vec![vk::PushConstantRange::builder()
                .stage_flags(push_bind_flag.0)
                .offset(0)
                .size(push_constant_size)
                .build()]
        };

        let layout_info =
            vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&push_constant_ranges);
        let layout = unsafe {
            device
                .device
                .create_pipeline_layout(&layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&config.input_assembly_info)
            .viewport_state(&config.viewport_info)
            .rasterization_state(&config.rasterization_info)
            .multisample_state(&config.multisample_info)
            .color_blend_state(&config.color_blend_info)
            .depth_stencil_state(&config.depth_stencil_info)
            .dynamic_state(&config.dynamic_state_info)
            .layout(layout)
            .render_pass(*render_pass)
            .subpass(config.subpass);

        let graphics_pipeline = unsafe {
            device
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|e| log::error!("Unable to create graphics pipeline: {:?}", e))
                .unwrap()[0]
        };

        unsafe {
            device
                .device
                .destroy_shader_module(vertex_shader_module, None);
            device
                .device
                .destroy_shader_module(fragment_shader_module, None);
        };

        Pipeline {
            device,
            graphics_pipeline,
            layout,
            models: Vec::new(),
        }
    }

    /// Creates a new [`Model`] with the same [`Vertex`] and [`PushConstantData`] as the [`Pipeline`].
    ///
    /// The [`PushConstantData`] on the new [`Model`] will be set to None. See also [`Model::new`]
    pub fn create_model(
        &mut self,
        vertices: Vec<V>,
        indices: Vec<u32>,
    ) -> Rc<RefCell<Model<P, V>>> {
        let model = Rc::new(RefCell::new(Model::new(
            self.device.clone(),
            vertices,
            indices,
        )));
        self.models.push(model.clone());
        model
    }

    /// Creates a new [`Model`] with the same [`Vertex`] and [`PushConstantData`] as the [`Pipeline`]
    /// and will set the [`PushConstantData`] on the new [`Model`].
    ///
    /// See also [`Model::new_with_push`].
    pub fn create_model_with_push(
        &mut self,
        vertices: Vec<V>,
        indices: Vec<u32>,
        push_constants: P,
    ) -> Rc<RefCell<Model<P, V>>> {
        let model = Rc::new(RefCell::new(Model::new_with_push(
            self.device.clone(),
            vertices,
            indices,
            push_constants,
        )));
        self.models.push(model.clone());
        model
    }

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

impl<P, V> RenderPipeline for Pipeline<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    /// Draws al the [`Model`]s in the [`Pipeline`] to the command buffer
    fn draw(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );
        };

        for model in self.models.iter() {
            model.borrow().draw(command_buffer, self.layout);
        }
    }
}

impl<P, V> Drop for Pipeline<P, V>
where
    P: PushConstantData,
    V: Vertex,
{
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.layout, None);
        };
    }
}
