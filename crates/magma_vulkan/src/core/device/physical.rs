use ash::vk;

use crate::{
    core::{device::QueueFamily, instance::Instance},
    utils,
};

#[derive(thiserror::Error, Debug)]
pub enum PhysicalDeviceError {
    #[error("There are no Vulkan capable devices on your machine")]
    NoPhysicalDevices,
    #[error("Failed to find a physical device that matched the requirements")]
    NoSuitableDevice,
    #[error("Not all queue families have an index")]
    IncompleteQueueFamilies,
}

pub enum PhysicalDeviceType {
    CPU,
    IntegratedGPU,
    DiscreteGPU,
    VirtualGPU,
    Other,
}

pub struct PhysicalDeviceBuilder {
    queue_families: Vec<QueueFamily>,
    preferred_type: PhysicalDeviceType,
}

impl PhysicalDeviceBuilder {
    pub fn new() -> PhysicalDeviceBuilder {
        PhysicalDeviceBuilder {
            queue_families: Vec::new(),
            preferred_type: PhysicalDeviceType::DiscreteGPU,
        }
    }

    pub fn add_queue_family(mut self, family: QueueFamily) -> PhysicalDeviceBuilder {
        self.queue_families.push(family);
        self
    }

    pub fn preferred_type(mut self, ty: PhysicalDeviceType) -> PhysicalDeviceBuilder {
        self.preferred_type = ty;
        self
    }

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
            queue_families: self.queue_families,
            properties,
            features,
            memory_properties,
            handle,
        })
    }
}

impl PhysicalDeviceBuilder {
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

    fn is_device_suitable(
        &mut self,
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<bool, PhysicalDeviceError> {
        PhysicalDeviceBuilder::find_queue_families(instance, device, &mut self.queue_families)?;
        Ok(true)
    }

    fn find_queue_families(
        instance: &Instance,
        device: vk::PhysicalDevice,
        queue_families: &mut [QueueFamily],
    ) -> Result<(), PhysicalDeviceError> {
        let device_queue_families = unsafe {
            instance
                .vk_handle()
                .get_physical_device_queue_family_properties(device)
        };

        for queue_family in queue_families.iter_mut() {
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

        if let Some(_) = queue_families.iter().find(|family| family.index.is_none()) {
            Err(PhysicalDeviceError::IncompleteQueueFamilies)
        } else {
            Ok(())
        }
    }
}

pub struct PhysicalDevice {
    queue_families: Vec<QueueFamily>,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,

    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn builder() -> PhysicalDeviceBuilder {
        PhysicalDeviceBuilder::new()
    }
}

impl PhysicalDevice {
    pub fn vk_handle(&self) -> vk::PhysicalDevice {
        self.handle
    }

    pub fn queue_families(&self) -> &[QueueFamily] {
        &self.queue_families
    }

    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }
}
