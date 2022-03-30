use ash::vk;
use std::{ffi::CStr, path::Path, rc::Rc};

use crate::{device::Device, mesh::Vertex};

pub struct Shader {
    pub file: String,
    pub entry_point: String,
    pub stage: vk::ShaderStageFlags,
}

impl Shader {
    pub const VERTEX: vk::ShaderStageFlags = vk::ShaderStageFlags::VERTEX;
    pub const FRAGMENT: vk::ShaderStageFlags = vk::ShaderStageFlags::FRAGMENT;
}

/// Allows for a struct to be passed to a [`Pipeline`] as a Vulkan push constant
pub trait PushConstantData {
    /// Converts [`PushConstantData`] to an array of bytes
    fn as_bytes(&self) -> &[u8];
}

pub struct PushConstant {
    pub stage: vk::ShaderStageFlags,
    pub offset: usize,
    pub size: usize,
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

pub struct Pipeline {
    /// Handle to the [`Device`] being used to draw
    device: Rc<Device>,
    /// Handle to the Vulkan pipeline that can be bound to draw graphics
    pub graphics_pipeline: vk::Pipeline,
    /// Handle to the layout of the pipeline
    pub layout: vk::PipelineLayout,
}

impl Pipeline {
    /// Creates a new [`Pipeline`].
    pub fn new<V>(
        device: Rc<Device>,
        config: PipelineConfigInfo,
        render_pass: &vk::RenderPass,
        shaders: &[Shader],
        push_constants: &[PushConstant],
    ) -> Pipeline
    where
        V: Vertex,
    {
        let mut shader_modules: Vec<vk::ShaderModule> = Vec::with_capacity(shaders.len());
        let mut shader_stages: Vec<vk::PipelineShaderStageCreateInfo> =
            Vec::with_capacity(shaders.len());

        for shader in shaders.iter() {
            let module = Pipeline::create_shader_module(device.vk(), Path::new(&shader.file));
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
        for push_constant in push_constants.iter() {
            push_constant_ranges.push(
                vk::PushConstantRange::builder()
                    .stage_flags(push_constant.stage)
                    .offset(push_constant.offset as u32)
                    .size(push_constant.size as u32)
                    .build(),
            );
        }

        let layout_info =
            vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&push_constant_ranges);
        let layout = unsafe {
            device
                .vk()
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
                device.vk().destroy_shader_module(module, None);
            };
        }

        Pipeline {
            device,
            graphics_pipeline,
            layout,
        }
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

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk()
                .destroy_pipeline(self.graphics_pipeline, None);
            self.device.vk().destroy_pipeline_layout(self.layout, None);
        };
    }
}
