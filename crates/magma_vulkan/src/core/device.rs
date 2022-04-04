use ash::vk;

mod physical;
pub use physical::{PhysicalDevice, PhysicalDeviceBuilder, PhysicalDeviceType};

#[derive(Clone, Copy, Debug)]
pub enum Queue {
    Graphics,
    Compute,
    Transfer,
    Sparse,
    Protected,
    VideoDecode,
    VideoEncode,
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
    ty: Queue,
    index: Option<u32>,
}

impl QueueFamily {
    pub fn new(ty: Queue) -> QueueFamily {
        QueueFamily { ty, index: None }
    }
}
