use ash::vk;
use std::rc::Rc;

use crate::device::LogicalDevice;

pub struct Framebuffer {
    device: Rc<LogicalDevice>,
    handle: vk::Framebuffer,
}

impl Framebuffer {
    pub fn new(
        device: Rc<LogicalDevice>,
        render_pass: vk::RenderPass,
        image_view: &vk::ImageView,
        extent: &vk::Extent2D,
    ) -> Framebuffer {
        let attachments = [image_view.clone()];
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let handle = unsafe {
            device
                .vk_handle()
                .create_framebuffer(&create_info, None)
                .expect("Failed to create framebuffer")
        };

        Framebuffer { device, handle }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .vk_handle()
                .destroy_framebuffer(self.handle, None);
        };
    }
}
