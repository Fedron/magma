use ash::vk;
use std::rc::Rc;

use crate::model::Vertex;

use super::device::Device;

#[repr(align(16))]
#[derive(Clone, Copy, Debug)]
pub struct Align16<T>(pub T);

pub struct PushConstants {
    pub offset: Align16<cgmath::Vector2<f32>>,
}

impl PushConstants {
    pub unsafe fn as_bytes(&self) -> &[u8] {
        let size_in_bytes = std::mem::size_of::<Self>();
        let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
        let ptr = self as *const Self as *const u8;
        std::slice::from_raw_parts(ptr, size_in_u8)
    }
}

pub struct Pipeline {
    /// Handle to the device this pipeline belongs to
    device: Rc<Device>,
    /// Handle to the Vulkan pipeline created
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipeline.html
    pub graphics_pipeline: vk::Pipeline,
    /// Handle to the pipeline layout for the current graphics pipeline
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineLayout.html
    pub layout: vk::PipelineLayout,

    /// The shader used by the graphics pipeline
    ///
    /// Should be compiled from a rust-gpu shader crate, and hence contain both an entry point for the vertex and
    /// fragment shader in one shader module
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkShaderModule.html
    shader_module: vk::ShaderModule,
}

impl Pipeline {
    /// Creates a new graphics pipeline for a device
    pub fn new(device: Rc<Device>, shader: &'static str, render_pass: vk::RenderPass) -> Pipeline {
        let code = ash::util::read_spv(
            &mut std::fs::File::open("shaders/simple.vert.spv").expect("Failed to open file"),
        )
        .expect("Failed to read spv");

        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);
        let vertex_shader_module = unsafe {
            device
                .device
                .create_shader_module(&create_info, None)
                .expect("Failed to create vert shader module")
        };

        // Compile shaders
        let shader_module = Pipeline::create_shader_module(&device.as_ref().device, shader);
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vertex_shader_module)
                .name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0") })
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .name(unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(b"main_fs\0") })
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        // Create the graphics pipeline
        let attribute_descriptions = Vertex::get_attribute_descriptions();
        let binding_descriptions = Vertex::get_binding_descriptions();
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&attribute_descriptions)
            .vertex_binding_descriptions(&binding_descriptions);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&[])
            .scissor_count(1)
            .viewports(&[])
            .viewport_count(1);

        let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL)
            .rasterizer_discard_enable(false)
            .depth_bias_clamp(0.0)
            .depth_bias_constant_factor(0.0)
            .depth_bias_enable(false)
            .depth_bias_slope_factor(0.0);

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .sample_mask(&[])
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false);

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0)
            .write_mask(0)
            .reference(0)
            .build();

        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state);

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state_enables = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state_enables);

        let push_constant_ranges = [vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<PushConstants>() as u32)
            .build()];

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let graphics_pipeline_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .dynamic_state(&dynamic_state_info)
            .build()];

        let graphics_pipeline = unsafe {
            device
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphics_pipeline_infos,
                    None,
                )
                .expect("Failed to create graphics pipeline")[0]
        };

        unsafe {
            device.device.destroy_shader_module(vertex_shader_module, None);
        };

        Pipeline {
            device,
            graphics_pipeline,
            layout: pipeline_layout,
            shader_module,
        }
    }

    /// Helper constructor that creates a new shader module from a rust-gpu crate
    fn create_shader_module(device: &ash::Device, shader_crate: &'static str) -> vk::ShaderModule {
        // FIXME: This assumes we are running an example from 'target/debug/examples'
        let mut shader_crate_path = std::env::current_exe().unwrap();
        shader_crate_path.pop();
        shader_crate_path.pop();
        shader_crate_path.pop();
        shader_crate_path.push("spirv-unknown-vulkan1.1");
        shader_crate_path.push("release");
        shader_crate_path.push("deps");
        shader_crate_path.push(format!(
            "{}.spv.dir\\module",
            shader_crate.replace("-", "_")
        ));
        let shader_code =
            ash::util::read_spv(&mut std::fs::File::open(shader_crate_path).unwrap()).unwrap();

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
                .destroy_shader_module(self.shader_module, None);
            self.device
                .device
                .destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.layout, None);
        };
    }
}
