use crate::{debugger::Debugger, instance::Instance, surface::Surface};
use ash::vk;

pub struct LogicalDevice<'a> {
    handle: vk::Device,
    physical_device: &'a PhysicalDevice,
    instance: &'a Instance,

    surface: Surface,
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
    present_queue: vk::Queue,

    debugger: Option<Debugger>,
}

pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,

    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}
