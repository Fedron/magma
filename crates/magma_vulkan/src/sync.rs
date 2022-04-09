//! This module provides wrappers around Vulkan synchronization objects

use std::rc::Rc;
use ash::vk;

use crate::core::device::{LogicalDevice, LogicalDeviceError};

/// Wraps a Vulkan Semaphore
#[derive(Clone)]
pub struct Semaphore {
    /// Opaque handle to Vulkan semaphore
    handle: vk::Semaphore,
    /// Logical device this semaphore belongs to
    device: Rc<LogicalDevice>,
}

impl Semaphore {
    /// Creates a new Semaphore on the device
    ///
    /// # Errors
    /// Vulkan may return an error when trying to create the semaphore, this error will be
    /// forwarded as a [LogicalDeviceError::Other]. Possible errors include:
    /// - [VulkanError::OutOfHostMemory]
    /// - [VulkanError::OutOFDeviceMemory]
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
    /// Returns a the handle to the Vulkan Semaphore
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

/// Wraps a Vulkan Fence
#[derive(Clone)]
pub struct Fence {
    /// Opaque handle to Vulkan Fence
    handle: vk::Fence,
    /// Logical device this fence belongs to
    device: Rc<LogicalDevice>,
}

impl Fence {
    /// Creates a new Fence
    ///
    /// # Errors
    /// Vulkan may return an error when trying to create the Fence, this error will be forwarded as
    /// a [LogicalDeviceError::Other]. Possible errors include:
    /// - [VulkanError::OutOfHostMemory]
    /// - [VulkanError::OutOFDeviceMemory]
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
    /// Returns a handle to the Vulkan Fence
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
