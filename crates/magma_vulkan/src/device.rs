use crate::{
    debugger::{Debugger, ENABLE_VALIDATION_LAYERS, VALIDATION_LAYERS},
    instance::Instance,
    surface::Surface,
    utils,
};
use ash::vk;

const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

pub struct LogicalDevice {
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    transfer_queue: vk::Queue,

    _debugger: Option<Debugger>,
    physical_device: PhysicalDevice,
    surface: Surface,
    instance: Instance,
    handle: ash::Device,
}

impl LogicalDevice {
    pub fn new(
        instance: Instance,
        surface: Surface,
        physical_device: PhysicalDevice,
    ) -> LogicalDevice {
        use std::collections::HashSet;
        use std::ffi::CString;

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(physical_device.indices.graphics_family.unwrap());
        unique_queue_families.insert(physical_device.indices.present_family.unwrap());
        unique_queue_families.insert(physical_device.indices.transfer_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        for &queue_family in unique_queue_families.iter() {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family)
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

        let device_extension_names_cstring: Vec<CString> = DEVICE_EXTENSIONS
            .iter()
            .map(|&extension| CString::new(extension).unwrap())
            .collect();

        let device_extension_names_ptr: Vec<*const i8> = device_extension_names_cstring
            .iter()
            .map(|t| t.as_ptr())
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(&physical_device.features)
            .enabled_layer_names(&required_validation_layers)
            .enabled_extension_names(&device_extension_names_ptr)
            .build();

        let handle = unsafe {
            instance
                .vk_handle()
                .create_device(physical_device.vk_handle(), &create_info, None)
                .expect("Failed to create Vulkan logical device")
        };

        let graphics_queue =
            unsafe { handle.get_device_queue(physical_device.indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { handle.get_device_queue(physical_device.indices.present_family.unwrap(), 0) };
        let transfer_queue =
            unsafe { handle.get_device_queue(physical_device.indices.transfer_family.unwrap(), 0) };

        let debugger: Option<Debugger> = if ENABLE_VALIDATION_LAYERS {
            log::debug!("Created Vulkan debugger");
            Some(Debugger::new(instance.entry(), instance.vk_handle()))
        } else {
            None
        };

        LogicalDevice {
            handle,
            physical_device,
            instance,

            _debugger: debugger,
            surface,
            graphics_queue,
            present_queue,
            transfer_queue,
        }
    }
}

impl LogicalDevice {
    pub fn vk_handle(&self) -> &ash::Device {
        &self.handle
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn transfer_queue(&self) -> vk::Queue {
        self.transfer_queue
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        };
    }
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

    indices: QueueFamilyIndices,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &ash::Instance, surface: &Surface) -> PhysicalDevice {
        let handle = PhysicalDevice::pick_device(instance, surface);
        let indices = PhysicalDevice::find_queue_family(instance, handle, surface);
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

            indices,
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
