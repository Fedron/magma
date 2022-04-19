use ash::vk;

use crate::{
    core::{device::QueueFamily, instance::Instance},
    utils, VulkanError,
};

use super::{DeviceExtension, QueueFlags};

/// Errors that the physical device can throw
#[derive(thiserror::Error, Debug)]
pub enum PhysicalDeviceError {
    #[error("There are no Vulkan capable devices on your machine")]
    NoPhysicalDevices,
    #[error("Failed to find a physical device that matched the requirements")]
    NoSuitableDevice,
    #[error("Not all queue families have an index")]
    IncompleteQueueFamilies,
    #[error("The physical device doesn't support some (or all) of the required extensions")]
    UnsupportedExtensions(Vec<String>),
    #[error(transparent)]
    Other(#[from] VulkanError),
}

/// Possible physical device types
pub enum PhysicalDeviceType {
    CPU,
    IntegratedGPU,
    DiscreteGPU,
    VirtualGPU,
    Other,
}

/// Wraps the steps required to create a [PhysicalDevice]
pub struct PhysicalDeviceBuilder {
    /// Queue families to create the physical device with
    queue_families: Vec<QueueFamily>,
    /// Type of device to use, if found
    // FIXME: Not being taken into account
    preferred_type: PhysicalDeviceType,
    /// Device extensions to enable on the physical device
    device_extensions: Vec<DeviceExtension>,
}

impl PhysicalDeviceBuilder {
    /// Creates a new [PhysicalDeviceBuilder]
    pub fn new() -> PhysicalDeviceBuilder {
        PhysicalDeviceBuilder {
            queue_families: Vec::new(),
            preferred_type: PhysicalDeviceType::DiscreteGPU,
            device_extensions: Vec::new(),
        }
    }

    /// Adds a queue family to create the physical device with
    pub fn add_queue_family(mut self, family: QueueFamily) -> PhysicalDeviceBuilder {
        self.queue_families.push(family);
        self
    }

    /// Sets the preferred type of physical device to use
    pub fn preferred_type(mut self, ty: PhysicalDeviceType) -> PhysicalDeviceBuilder {
        self.preferred_type = ty;
        self
    }

    /// Sets the device extensions to create the physical device with
    pub fn device_extensions(mut self, extensions: &[DeviceExtension]) -> PhysicalDeviceBuilder {
        self.device_extensions = extensions.to_vec();
        self
    }

    /// Creates a [PhysicalDevice]
    pub fn build(mut self, instance: &Instance) -> Result<PhysicalDevice, PhysicalDeviceError> {
        let handle = self.pick_physical_device(instance)?;

        let properties = unsafe { instance.vk_handle().get_physical_device_properties(handle) };
        let features = unsafe { instance.vk_handle().get_physical_device_features(handle) };
        let memory_properties = unsafe {
            instance
                .vk_handle()
                .get_physical_device_memory_properties(handle)
        };

        log::info!(
            "Using {} ({})",
            utils::char_array_to_string(&properties.device_name),
            match properties.device_type {
                vk::PhysicalDeviceType::CPU => "Cpu",
                vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
                vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
                vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
                _ => "Unknown",
            }
        );

        Ok(PhysicalDevice {
            extensions: self.device_extensions,
            queue_families: self.queue_families,

            properties,
            features,
            memory_properties,

            handle,
        })
    }
}

impl PhysicalDeviceBuilder {
    /// Finds the first physical device that matches the requirements of the
    /// [PhysicalDeviceBuilder].
    ///
    /// If no device matches the requirements then [PhysicalDeviceError::NoSuitableDevice] is
    /// returned.
    fn pick_physical_device(
        &mut self,
        instance: &Instance,
    ) -> Result<vk::PhysicalDevice, PhysicalDeviceError> {
        let physical_devices = unsafe {
            instance
                .vk_handle()
                .enumerate_physical_devices()
                .map_err(|_| PhysicalDeviceError::NoPhysicalDevices)?
        };

        let mut chosen_device: Option<vk::PhysicalDevice> = None;
        for &physical_device in physical_devices.iter() {
            if self.is_device_suitable(instance, physical_device)? {
                chosen_device = Some(physical_device);
                break;
            }
        }

        match chosen_device {
            Some(device) => Ok(device),
            None => Err(PhysicalDeviceError::NoSuitableDevice),
        }
    }

    /// Checks wether the Vulkan physical device contains the requried queue families and supports
    /// the required device extensions.
    fn is_device_suitable(
        &mut self,
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<bool, PhysicalDeviceError> {
        self.find_queue_families(instance, device)?;
        self.check_device_extension_support(instance, device)?;

        Ok(true)
    }

    /// Goes through all the queue families of the physical device, finding the ones required by
    /// the [PhysicalDeviceBuilder].
    ///
    /// If not all the required queue families were found
    /// [PhysicalDeviceError::IncompleteQueueFamilies] is returned.
    fn find_queue_families(
        &mut self,
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<(), PhysicalDeviceError> {
        let device_queue_families = unsafe {
            instance
                .vk_handle()
                .get_physical_device_queue_family_properties(device)
        };

        for queue_family in self.queue_families.iter_mut() {
            for (index, device_queue_family) in device_queue_families.iter().enumerate() {
                if device_queue_family.queue_count > 0
                    && device_queue_family
                        .queue_flags
                        .contains(queue_family.ty.into())
                {
                    queue_family.index = Some(index as u32);
                    break;
                }
            }
        }

        if let Some(_) = self
            .queue_families
            .iter()
            .find(|family| family.index.is_none())
        {
            Err(PhysicalDeviceError::IncompleteQueueFamilies)
        } else {
            Ok(())
        }
    }

    /// Checks wether the physical device supports all the required device extensions in the
    /// [PhysicalDeviceBuilder].
    fn check_device_extension_support(
        &self,
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<(), PhysicalDeviceError> {
        let available_extension_names = unsafe {
            instance
                .vk_handle()
                .enumerate_device_extension_properties(device)
                .map_err(|err| PhysicalDeviceError::Other(err.into()))?
        };

        let is_missing_extensions = utils::contains_required(
            &available_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &self
                .device_extensions
                .iter()
                .map(|&extension| extension.to_string())
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            Err(PhysicalDeviceError::UnsupportedExtensions(
                is_missing_extensions.1,
            ))
        } else {
            Ok(())
        }
    }
}

/// Wraps a Vulkan physical device and its capabilities
pub struct PhysicalDevice {
    /// List of all enabled device extensions
    extensions: Vec<DeviceExtension>,
    /// List of queue families the physical device supports (that it was created with)
    queue_families: Vec<QueueFamily>,

    /// Vulkan physical device properties
    properties: vk::PhysicalDeviceProperties,
    /// Vulkan physical device features
    features: vk::PhysicalDeviceFeatures,
    /// Vulkan physical device memory properties
    memory_properties: vk::PhysicalDeviceMemoryProperties,

    /// Opaque handle to Vulkan physical device
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    /// Creates a new [PhysicalDeviceBuilder]
    pub fn builder() -> PhysicalDeviceBuilder {
        PhysicalDeviceBuilder::new()
    }
}

impl PhysicalDevice {
    /// Returns the Vulkan handle to the physic device
    pub(crate) fn vk_handle(&self) -> vk::PhysicalDevice {
        self.handle
    }

    /// Returns a list of all the enabled device extensions
    pub fn enabled_extensions(&self) -> &[DeviceExtension] {
        &self.extensions
    }

    /// Returns a list of all the queue families the device was created with
    pub fn queue_families(&self) -> &[QueueFamily] {
        &self.queue_families
    }

    /// Looks through the device's queue family and tries to find a family that matches the type
    /// `ty`.
    ///
    /// If no queue families have the type `ty`, then `None` is returned.
    pub fn queue_family(&self, ty: QueueFlags) -> Option<&QueueFamily> {
        self.queue_families.iter().find(|family| family.ty.contains(ty))
    }

    /// Returns the Vulkan physical device properties
    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    /// Returns the Vulkan physical device features
    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    /// Returns the Vulkan physical device memory properties
    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }
}
