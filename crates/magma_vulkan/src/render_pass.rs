use ash::vk;
use std::rc::Rc;

use crate::device::LogicalDevice;

pub struct RenderPass {
    device: Rc<LogicalDevice>,
    handle: vk::RenderPass,
}

impl RenderPass {
    pub fn new(
        device: Rc<LogicalDevice>,
        color_format: &vk::Format,
        depth_format: &vk::Format,
    ) -> RenderPass {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(*color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(*depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let render_pass_attachments = [color_attachment, depth_attachment];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let handle = unsafe {
            device
                .vk_handle()
                .create_render_pass(&create_info, None)
                .expect("Failed to create render pass")
        };

        RenderPass { device, handle }
    }
}

impl RenderPass {
    pub fn vk_handle(&self) -> vk::RenderPass {
        self.handle
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_render_pass(self.handle, None);
        };
    }
}
