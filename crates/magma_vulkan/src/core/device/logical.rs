use std::ffi::CString;
use ash::vk;

use super::{PhysicalDevice, Queue};
use crate::{
    core::{
        device::QueueHandle,
        instance::Instance,
    },
    sync::Fence,
    VulkanError,
};

/// Errors that could be thrown by the logical device
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

/// Wraps a Vulkan logical devcie, allowing you to interface with a [PhysicalDevice]
pub struct LogicalDevice {
    /// Vulkan handles to all of the queues of the [PhysicalDevice]
    queues: Vec<QueueHandle>,

    /// [PhysicalDevice] this logical device interfaces with
    physical_device: PhysicalDevice,
    /// Opaque handle to Vulkan logical device
    handle: ash::Device,
    /// [Instance] this device is using
    instance: Instance,
}

impl LogicalDevice {
    /// Creates a new [LogicalDevice]
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

        let required_validation_layers_raw: Vec<CString> = instance.debug_layers().iter().map(|&layer| Into::<CString>::into(layer)).collect();
        let required_validation_layers: Vec<*const i8> = required_validation_layers_raw.iter().map(|layer| layer.as_ptr()).collect();

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

        let mut queues: Vec<QueueHandle> = Vec::new();
        for queue_family in physical_device.queue_families().iter() {
            queues.push(QueueHandle {
                handle: unsafe { handle.get_device_queue(queue_family.index.unwrap(), 0) },
                ty: queue_family.ty,
            });
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
    /// Returns the Vulkan handle to the logical device
    pub(crate) fn vk_handle(&self) -> &ash::Device {
        &self.handle
    }

    /// Returns a list of [QueueHandles][QueueHandle] to each of the
    /// [PhysicalDevice's][PhysicalDevice] queue families
    pub fn queues(&self) -> &[QueueHandle] {
        &self.queues
    }

    /// Returns a [QueueHandle] to a queue with the type `ty`
    pub fn queue(&self, ty: Queue) -> Option<&QueueHandle> {
        self.queues.iter().find(|queue| queue.ty == ty)
    }

    /// Returns the [PhysicalDevcie] this logical device is interfacing with
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    /// Returns the [Instance] this logical device was created with
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

impl LogicalDevice {
    /// Waits for the [PhysicalDevice] to idle/stop using resources
    pub fn wait_for_idle(&self) -> Result<(), LogicalDeviceError> {
        Ok(unsafe {
            self.handle
                .device_wait_idle()
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        })
    }

    /// Waits the logical device until `wait_all` fences are signaled by the host
    pub fn wait_for_fences(
        &self,
        fences: &[&Fence],
        wait_all: bool,
        timeout: u64,
    ) -> Result<(), LogicalDeviceError> {
        if fences.len() == 0 {
            return Ok(());
        }

        let wait_fences: Vec<vk::Fence> = fences.iter().map(|&fence| fence.vk_handle()).collect();
        unsafe {
            self.handle
                .wait_for_fences(&wait_fences, wait_all, timeout)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        Ok(())
    }

    /// Resest the Vulkan fences
    pub fn reset_fences(&self, fences: &[&Fence]) -> Result<(), LogicalDeviceError> {
        let fences: Vec<vk::Fence> = fences.iter().map(|&fence| fence.vk_handle()).collect();
        unsafe {
            self.handle
                .reset_fences(&fences)
                .map_err(|err| LogicalDeviceError::Other(err.into()))?
        };

        Ok(())
    }

    /// Creates a Vulkan image and Vulkan device memory
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

    /// Finds a memory type on the [PhysicalDevice] that matches the `type_filter` and
    /// `required_properties`.
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

    /// Finds a supported format on the [PhysicalDevice] from the list of `candidates`.
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
