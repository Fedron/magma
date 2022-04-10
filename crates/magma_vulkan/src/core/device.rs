//! This module wraps Vulkan physical and logical devices

use std::fmt::Display;
use ash::vk;

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

/// Represents Vulkan queue flags
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Queue {
    Graphics,
    Compute,
    Transfer,
    Sparse,
    Protected,
    VideoDecode,
    VideoEncode,
}

impl Display for Queue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Queue::Graphics => write!(f, "Graphics"),
            Queue::Compute => write!(f, "Compute"),
            Queue::Transfer => write!(f, "Transfer"),
            Queue::Sparse => write!(f, "Sparse"),
            Queue::Protected => write!(f, "Protected"),
            Queue::VideoDecode => write!(f, "Video Decode"),
            Queue::VideoEncode => write!(f, "Video Encode"),
        }
    }
}

impl Into<vk::QueueFlags> for Queue {
    fn into(self) -> vk::QueueFlags {
        match self {
            Queue::Graphics => vk::QueueFlags::GRAPHICS,
            Queue::Compute => vk::QueueFlags::COMPUTE,
            Queue::Transfer => vk::QueueFlags::TRANSFER,
            Queue::Sparse => vk::QueueFlags::SPARSE_BINDING,
            Queue::Protected => vk::QueueFlags::PROTECTED,
            Queue::VideoDecode => vk::QueueFlags::VIDEO_DECODE_KHR,
            Queue::VideoEncode => vk::QueueFlags::VIDEO_ENCODE_KHR,
        }
    }
}

/// Wraps a Vulkan Queue
pub struct QueueHandle {
    /// Opaque handle to Vulkan Queue
    pub(crate) handle: vk::Queue,
    /// Type of the Queue
    pub ty: Queue,
}

/// Wraps the index of a given Queue type
#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    /// Type of the queue family
    pub ty: Queue,
    /// Index on the device where the queue is
    pub index: Option<u32>,
}

impl QueueFamily {
    /// Creates a new [QueueFamily] of the type setting the index to `None`
    pub fn new(ty: Queue) -> QueueFamily {
        QueueFamily { ty, index: None }
    }
}
