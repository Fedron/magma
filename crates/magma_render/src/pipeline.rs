use ash::vk;
use std::{ffi::CString, path::Path, rc::Rc};

use crate::{device::Device, render_system::Vertex};

pub struct PipelineConfigInfo {
    viewport_info: vk::PipelineViewportStateCreateInfo,
    input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    multisample_info: vk::PipelineMultisampleStateCreateInfo,
    color_blend_attachment: Rc<vk::PipelineColorBlendAttachmentState>,
    color_blend_info: Rc<vk::PipelineColorBlendStateCreateInfo>,
    depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    dynamic_state_enables: Vec<vk::DynamicState>,
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

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&[color_blend_attachment])
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
            color_blend_attachment: Rc::new(color_blend_attachment),
            color_blend_info: Rc::new(color_blend_info),
            depth_stencil_info,
            dynamic_state_enables,
            dynamic_state_info,
            subpass: 0,
        }
    }
}

pub struct Pipeline {
    device: Rc<Device>,
    pub graphics_pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl Pipeline {
    pub fn new<V>(
        device: Rc<Device>,
        config: PipelineConfigInfo,
        render_pass: &vk::RenderPass,
        layout: vk::PipelineLayout,
        vertex_shader_file: &Path,
        fragment_shader_file: &Path,
    ) -> Pipeline
    where
        V: Vertex,
    {
        let vertex_shader_module =
            Pipeline::create_shader_module(&device.as_ref().device, vertex_shader_file);
        let fragment_shader_module =
            Pipeline::create_shader_module(&device.as_ref().device, fragment_shader_file);

        // Compile shaders
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
        }
    }

    pub fn bind(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.graphics_pipeline,
            );
        };
    }

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
                .device
                .destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.layout, None);
        };
    }
}
