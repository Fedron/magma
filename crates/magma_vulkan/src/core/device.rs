use std::fmt::Display;

use ash::vk;

mod logical;
mod physical;

pub use logical::{LogicalDevice, LogicalDeviceError};
pub use physical::{
    PhysicalDevice, PhysicalDeviceBuilder, PhysicalDeviceError, PhysicalDeviceType,
};

#[derive(Clone, Copy, PartialEq, Eq)]
/// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#extension-appendices-list
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

#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    pub ty: Queue,
    pub index: Option<u32>,
}

impl QueueFamily {
    pub fn new(ty: Queue) -> QueueFamily {
        QueueFamily { ty, index: None }
    }
}
