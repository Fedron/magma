use crate::{instance::Instance, surface::Surface, utils};
use ash::vk;

const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

pub struct LogicalDevice<'a> {
    handle: vk::Device,
    physical_device: &'a PhysicalDevice,
    instance: &'a Instance,

    surface: Surface,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    transfer_queue: vk::Queue,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
            && self.present_family.is_some()
            && self.transfer_family.is_some()
    }
}

pub struct SwapchainSupportInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,

    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &ash::Instance, surface: &Surface) -> PhysicalDevice {
        let handle = PhysicalDevice::pick_device(instance, surface);
        let properties = unsafe { instance.get_physical_device_properties(handle) };
        let features = unsafe { instance.get_physical_device_features(handle) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(handle) };

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

        PhysicalDevice {
            handle,
            properties,
            features,
            memory_properties,
        }
    }

    fn pick_device(instance: &ash::Instance, surface: &Surface) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to get Vulkan capable physical devices")
        };

        let mut chosen_device: Option<vk::PhysicalDevice> = None;
        for &physical_device in physical_devices.iter() {
            if PhysicalDevice::is_suitable(instance, physical_device, surface) {
                chosen_device = Some(physical_device);
                break;
            }
        }

        match chosen_device {
            Some(device) => device,
            None => panic!("Failed to find a suitable device"),
        }
    }

    fn is_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> bool {
        PhysicalDevice::check_device_extension_support(instance, device);

        let indices = PhysicalDevice::find_queue_family(instance, device, surface);
        let swapchain_support = PhysicalDevice::query_swapchain_support(device, surface);

        indices.is_complete()
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty()
    }

    fn find_queue_family(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
            transfer_family: None,
        };

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0 {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    queue_family_indices.graphics_family = Some(index as u32);
                }

                if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                    queue_family_indices.transfer_family = Some(index as u32);
                }

                let has_present_support = unsafe {
                    surface
                        .surface()
                        .get_physical_device_surface_support(
                            device,
                            index as u32,
                            surface.vk_handle(),
                        )
                        .expect("Failed to get surface support for physical device")
                };
                if has_present_support {
                    queue_family_indices.present_family = Some(index as u32);
                }
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }

        queue_family_indices
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) {
        let available_extension_names = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get instance device properties")
        };

        let is_missing_extensions = utils::contains_required(
            &available_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &DEVICE_EXTENSIONS
                .iter()
                .map(|&extension| extension.to_string())
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            panic!("Missing extensions, see above")
        }
    }

    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> SwapchainSupportInfo {
        unsafe {
            let capabilities = surface
                .surface()
                .get_physical_device_surface_capabilities(physical_device, surface.vk_handle())
                .expect("Failed to query for surface capabilities");

            let formats = surface
                .surface()
                .get_physical_device_surface_formats(physical_device, surface.vk_handle())
                .expect("Failed to query for surface formats");

            let present_modes = surface
                .surface()
                .get_physical_device_surface_present_modes(physical_device, surface.vk_handle())
                .expect("Failed to query for surface present modes");

            SwapchainSupportInfo {
                capabilities,
                formats,
                present_modes,
            }
        }
    }
}

impl PhysicalDevice {
    pub fn vk_handle(&self) -> vk::PhysicalDevice {
        self.handle
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
