use ash::vk;
use std::rc::Rc;

/// Wraps fixed function configuration for creating a Vulkan graphics pipeline
pub struct PipelineConfigInfo {
    pub viewport_info: vk::PipelineViewportStateCreateInfo,
    pub input_assembly_info: vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterization_info: vk::PipelineRasterizationStateCreateInfo,
    pub multisample_info: vk::PipelineMultisampleStateCreateInfo,
    pub color_blend_attachment: Rc<vk::PipelineColorBlendAttachmentState>,
    pub color_blend_info: vk::PipelineColorBlendStateCreateInfo,
    pub depth_stencil_info: vk::PipelineDepthStencilStateCreateInfo,
    pub dynamic_state_enables: Vec<vk::DynamicState>,
    pub dynamic_state_info: vk::PipelineDynamicStateCreateInfo,
    pub subpass: u32,
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
            color_blend_attachment,
            color_blend_info,
            depth_stencil_info,
            dynamic_state_enables,
            dynamic_state_info,
            subpass: 0,
        }
    }
}
