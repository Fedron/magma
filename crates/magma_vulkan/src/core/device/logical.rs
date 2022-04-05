use std::fmt::Display;

use ash::vk;

use super::PhysicalDevice;
use crate::{
    core::{
        debugger::{ENABLE_VALIDATION_LAYERS, VALIDATION_LAYERS},
        instance::Instance,
    },
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum LogicalDeviceError {
    #[error("Failed to create a logical device")]
    CantCreate(VulkanError),
}

/// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/html/vkspec.html#extension-appendices-list
pub enum DeviceExtension {
    Surface,
    Swapchain,
}

impl Display for DeviceExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceExtension::Surface => write!(f, "VK_KHR_surface\0"),
            DeviceExtension::Swapchain => write!(f, "VK_KHR_swapchain\0"),
        }
    }
}

pub struct LogicalDevice {
    queues: Vec<vk::Queue>,

    physical_device: PhysicalDevice,
    handle: ash::Device,
    instance: Instance,
}

impl LogicalDevice {
    pub fn new(
        instance: Instance,
        physical_device: PhysicalDevice,
        device_extensions: &[DeviceExtension],
    ) -> Result<LogicalDevice, LogicalDeviceError> {
        use std::collections::HashSet;
        use std::ffi::CStr;

        let mut unique_queue_indices = HashSet::new();
        for queue_family in physical_device.queue_families().iter() {
            unique_queue_indices.insert(queue_family.index.unwrap());
        }

        let queue_priorities = [1.0_f32];
        let mut queue_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        for &queue_index in unique_queue_indices.iter() {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_index)
                    .queue_priorities(&queue_priorities)
                    .build(),
            );
        }

        let required_validation_layers: Vec<*const i8> = if ENABLE_VALIDATION_LAYERS {
            VALIDATION_LAYERS
                .iter()
                .map(|layer| layer.as_ptr() as *const i8)
                .collect()
        } else {
            Vec::new()
        };

        let device_extensions: Vec<*const i8> = device_extensions
            .iter()
            .map(|extension| {
                unsafe { CStr::from_bytes_with_nul_unchecked(extension.to_string().as_bytes()) }
                    .as_ptr()
            })
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(physical_device.features())
            .enabled_layer_names(&required_validation_layers)
            .enabled_extension_names(&device_extensions);

        let handle = unsafe {
            instance
                .vk_handle()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .map_err(|err| LogicalDeviceError::CantCreate(err.into()))?
        };

        let mut queues: Vec<vk::Queue> = Vec::new();
        for queue_family in physical_device.queue_families().iter() {
            queues.push(unsafe { handle.get_device_queue(queue_family.index.unwrap(), 0) });
        }

        Ok(LogicalDevice {
            queues,
            physical_device,
            handle,
            instance,
        })
    }
}

impl LogicalDevice {
    pub(crate) fn vk_handle(&self) -> &ash::Device {
        &self.handle
    }
    pub fn queues(&self) -> &[vk::Queue] {
        &self.queues
    }

    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        };
    }
}
