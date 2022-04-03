use ash::vk;
use std::rc::Rc;

use crate::prelude::LogicalDevice;

pub struct Semaphore {
    device: Rc<LogicalDevice>,
    handle: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: Rc<LogicalDevice>) -> Semaphore {
        let create_info = vk::SemaphoreCreateInfo::default();

        let handle = unsafe {
            device
                .vk_handle()
                .create_semaphore(&create_info, None)
                .expect("Failed to create semaphore")
        };

        Semaphore { device, handle }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.vk_handle().destroy_semaphore(self.handle, None);
        };
    }
}

pub struct Fence {
    device: Rc<LogicalDevice>,
    handle: vk::Fence,
}

impl Fence {
    pub fn new(device: Rc<LogicalDevice>) -> Fence {
        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        let handle = unsafe {
            device
                .vk_handle()
                .create_fence(&create_info, None)
                .expect("Failed to create fence")
        };

        Fence { device, handle }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.vk_handle().destroy_fence(self.handle, None);
        };
    }
}
