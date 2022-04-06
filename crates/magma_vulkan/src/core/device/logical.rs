use std::ffi::CString;

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
    #[error("Failed to find a format from the candidates that is supported by the device")]
    NoSupportedFormat,
    #[error(
        "Failed to find a memory type that the device supports that matches caller's requirements"
    )]
    NoSupportedMemoryType,
    #[error(transparent)]
    Other(#[from] VulkanError),
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
    ) -> Result<LogicalDevice, LogicalDeviceError> {
        use std::collections::HashSet;

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

        let device_extensions: Vec<CString> = physical_device
            .enabled_extensions()
            .iter()
            .map(|extension| CString::new(extension.to_string()).expect("Failed to create CString"))
            .collect();

        let device_extensions_ptr: Vec<*const i8> = device_extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(physical_device.features())
            .enabled_layer_names(&required_validation_layers)
            .enabled_extension_names(&device_extensions_ptr);

        let handle = unsafe {
            instance
                .vk_handle()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .map_err(|err| {
                    println!("{:#?}", err);
                    LogicalDeviceError::CantCreate(err.into())
                })?
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

impl LogicalDevice {
    pub fn create_image(
        &self,
        create_info: &vk::ImageCreateInfo,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory), LogicalDeviceError> {
        let image = unsafe {
            self.handle
                .create_image(create_info, None)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        let memory_requirements = unsafe { self.handle.get_image_memory_requirements(image) };
        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(
                self.find_memory_type(memory_requirements.memory_type_bits, memory_properties)?,
            );

        let device_memory = unsafe {
            self.handle
                .allocate_memory(&allocate_info, None)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        unsafe {
            self.handle
                .bind_image_memory(image, device_memory, 0)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        Ok((image, device_memory))
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, LogicalDeviceError> {
        for (i, memory_type) in self
            .physical_device
            .memory_properties()
            .memory_types
            .iter()
            .enumerate()
        {
            if (type_filter & (1 << i)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return Ok(i as u32);
            }
        }

        Err(LogicalDeviceError::NoSupportedMemoryType)
    }

    pub fn find_supported_format(
        &self,
        candidates: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format, LogicalDeviceError> {
        for &format in candidates {
            let properties = unsafe {
                self.instance
                    .vk_handle()
                    .get_physical_device_format_properties(self.physical_device.vk_handle(), format)
            };

            if tiling == vk::ImageTiling::LINEAR
                && properties.linear_tiling_features.contains(features)
            {
                return Ok(format);
            } else if tiling == vk::ImageTiling::OPTIMAL
                && properties.optimal_tiling_features.contains(features)
            {
                return Ok(format);
            }
        }

        Err(LogicalDeviceError::NoSupportedFormat)
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        };
    }
}
