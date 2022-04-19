//! This module wraps Vulkan physical and logical devices

use ash::vk;
use bitflags::bitflags;
use std::fmt::Display;

mod logical;
mod physical;

pub use logical::{LogicalDevice, LogicalDeviceError};
pub use physical::{
    PhysicalDevice, PhysicalDeviceBuilder, PhysicalDeviceError, PhysicalDeviceType,
};

/// Vulkan device extensions that are supported my [`magma_vulkan`]
///
/// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#extension-appendices-list
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeviceExtension {
    Swapchain,
}

impl Display for DeviceExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceExtension::Swapchain => write!(f, "VK_KHR_swapchain"),
        }
    }
}

bitflags! {
    /// Represents Vulkan queue flags
    pub struct QueueFlags: u32 {
        const GRAPHICS = 0x1;
        const COMPUTE = 0x2;
        const TRANSFER = 0x4;
    }
}

impl Display for QueueFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Into<vk::QueueFlags> for QueueFlags {
    fn into(self) -> vk::QueueFlags {
        vk::QueueFlags::from_raw(self.bits())
    }
}

/// Wraps a Vulkan Queue
pub struct QueueHandle {
    /// Opaque handle to Vulkan Queue
    pub(crate) handle: vk::Queue,
    /// Type of the Queue
    pub ty: QueueFlags,
}

/// Wraps the index of a given Queue type
#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    /// Type of the queue family
    pub ty: QueueFlags,
    /// Index on the device where the queue is
    pub index: Option<u32>,
}

impl QueueFamily {
    /// Creates a new [QueueFamily] of the type setting the index to `None`
    pub fn new(ty: QueueFlags) -> QueueFamily {
        QueueFamily { ty, index: None }
    }
}
