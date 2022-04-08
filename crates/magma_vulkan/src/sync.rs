use std::rc::Rc;

use ash::vk;

use crate::core::device::{LogicalDevice, LogicalDeviceError};

#[derive(Clone)]
pub struct Semaphore {
    handle: vk::Semaphore,
    device: Rc<LogicalDevice>,
}

impl Semaphore {
    pub fn new(device: Rc<LogicalDevice>) -> Result<Semaphore, LogicalDeviceError> {
        let create_info = vk::SemaphoreCreateInfo::default();
        let handle = unsafe {
            device
                .vk_handle()
                .create_semaphore(&create_info, None)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        Ok(Semaphore { handle, device })
    }
}

impl Semaphore {
    pub(crate) fn vk_handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.vk_handle().destroy_semaphore(self.handle, None);
        };
    }
}

#[derive(Clone)]
pub struct Fence {
    handle: vk::Fence,
    device: Rc<LogicalDevice>,
}

impl Fence {
    pub fn new(device: Rc<LogicalDevice>) -> Result<Fence, LogicalDeviceError> {
        let create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let handle = unsafe {
            device
                .vk_handle()
                .create_fence(&create_info, None)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        Ok(Fence { handle, device })
    }
}

impl Fence {
    pub(crate) fn vk_handle(&self) -> vk::Fence {
        self.handle
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.vk_handle().destroy_fence(self.handle, None);
        };
    }
}
