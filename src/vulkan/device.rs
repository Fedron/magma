use ash::vk;
use std::ffi::CString;

use crate::utils;
use crate::utils::constants::*;

/// Contains information about a Vulkan physical device
struct PhysicalDeviceInfo {
    name: String,
    _device_id: u32,
    device_type: String,
    is_suitable: bool,
    handle: vk::PhysicalDevice,
}

impl std::fmt::Display for PhysicalDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.device_type)
    }
}

/// Wrapper struct with all the queue families required for the app
pub struct QueueFamilyIndices {
    /// Index of the graphics queue family
    pub graphics_family: Option<u32>,
    /// Index of the present queue family
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Returns whether or not all the queue family indices are present
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

/// Contains information on the features and properties of a swapchain
pub struct SwapchainSupportInfo {
    /// Various properties of the swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceCapabilitiesKHR.html
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    /// Supported color space and formats
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceFormatKHR.html
    pub formats: Vec<vk::SurfaceFormatKHR>,
    /// Supported present modes
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPresentModeKHR.html
    pub present_modes: Vec<vk::PresentModeKHR>,
}

/// Wraps the Vulkan steps to create a logical device
pub struct Device {
    /// Holds the loaded Vulkan library
    _entry: ash::Entry,
    /// Handle to the Vulkan instance
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkInstance.html
    pub instance: ash::Instance,

    /// Manages the debug_messenger
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    /// Handle to Vulkan debug messenger
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDebugUtilsMessengerEXT.html
    debug_messenger: vk::DebugUtilsMessengerEXT,

    /// Manages the Vulkan surface
    pub surface_loader: ash::extensions::khr::Surface,
    /// Handle to Vulkan surface
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceKHR.html
    pub surface: vk::SurfaceKHR,

    /// Handle to Vulkan physical device used to create this logical device
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html
    pub physical_device: vk::PhysicalDevice,
    /// Handle to Vulkan logical device
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDevice.html
    pub device: ash::Device,

    /// Handle to Vulkan queue used for graphics operations
    pub graphics_queue: vk::Queue,
    /// Handle to Vulkan queue used for presenting images
    pub present_queue: vk::Queue,

    /// Handle to Vulkan command pool that contains all our command buffers
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCommandPool.html
    pub command_pool: vk::CommandPool,
}

impl Device {
    /// Creates a new Vulkan instance and logical device
    pub fn new(window: &winit::window::Window) -> Device {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan library") };
        let instance = Device::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) =
            utils::debug::setup_debug_utils(&entry, &instance);

        let (surface_loader, surface) = Device::create_surface(&entry, &instance, &window);

        let physical_device = Device::pick_physical_device(&instance, &surface_loader, &surface);
        let (device, family_indices) =
            Device::create_logical_device(&instance, physical_device, &surface_loader, &surface);

        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let command_pool = Device::create_command_pool(&device, &family_indices);

        Device {
            _entry: entry,
            instance,

            debug_utils_loader,
            debug_messenger,

            surface_loader,
            surface,

            physical_device,
            device,

            graphics_queue,
            present_queue,

            command_pool,
        }
    }

    /// Helper constructor to create a Vulkan instance from a loaded Vulkan library
    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        let required_extension_names = utils::platforms::required_extension_names();
        if !Device::check_required_extensions(entry, &required_extension_names) {
            panic!("Missing extensions, see above");
        }

        if !utils::debug::check_validation_layer_support(
            entry,
            &utils::constants::VALIDATION_LAYERS,
        ) {
            panic!("Missing layers, see above");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Magma").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name);

        let enabled_layer_names = if utils::constants::ENABLE_VALIDATION_LAYERS {
            utils::constants::VALIDATION_LAYERS
                .iter()
                .map(|layer| layer.as_ptr() as *const i8)
                .collect::<Vec<*const i8>>()
        } else {
            Vec::new()
        };
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&required_extension_names)
            .enabled_layer_names(&enabled_layer_names);

        unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance")
        }
    }

    /// Checks if the Vulkan instance supports all the extensions we require
    ///
    /// Returns whether or not all required extensions are supported
    fn check_required_extensions(
        entry: &ash::Entry,
        required_extension_names: &Vec<*const i8>,
    ) -> bool {
        let supported_extension_names = entry
            .enumerate_instance_extension_properties(None)
            .expect("Failed to get instance extension properties");

        let is_missing_extensions = utils::contains_required(
            &supported_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &required_extension_names
                .iter()
                .map(|&extension| utils::char_ptr_to_string(extension))
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            panic!("Missing extensions, see above")
        }

        true
    }

    /// Helper constructor to create a platform-specific Vulkan surface
    ///
    /// Returns the surface loader and a handle to the created surface
    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> (ash::extensions::khr::Surface, vk::SurfaceKHR) {
        (
            ash::extensions::khr::Surface::new(entry, instance),
            unsafe {
                utils::platforms::create_surface(entry, instance, window)
                    .expect("Failed to create surface")
            },
        )
    }

    /// Helper constructor that finds a Vulkan physical device that matches the needs of the application, and returns it
    fn pick_physical_device(
        instance: &ash::Instance,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Vulkan physical devices")
        };

        let mut chosen_device: Option<PhysicalDeviceInfo> = None;
        for &physical_device in physical_devices.iter() {
            let physical_device_info = Device::is_physical_device_suitable(
                instance,
                physical_device,
                surface_loader,
                surface,
            );
            if physical_device_info.is_suitable {
                if chosen_device.is_none() {
                    chosen_device = Some(physical_device_info)
                }
            }
        }

        match chosen_device {
            Some(physical_device) => {
                log::info!("Using {}", physical_device);
                return physical_device.handle;
            }
            None => {
                log::error!("Failed to find a suitable GPU");
                panic!();
            }
        }
    }

    /// Checks a physical device for required features
    ///
    /// Returns whether or not the physical device is suitable
    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> PhysicalDeviceInfo {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            _ => "Unknown",
        };

        let indices = Device::find_queue_family(instance, physical_device, surface_loader, surface);

        let is_device_extensions_supported =
            Device::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = if is_device_extensions_supported {
            let swapchain_support =
                Device::query_swapchain_support(physical_device, surface_loader, surface);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        PhysicalDeviceInfo {
            name: utils::char_array_to_string(&device_properties.device_name),
            _device_id: device_properties.device_id,
            device_type: String::from(device_type),
            is_suitable: indices.is_complete()
                && is_device_extensions_supported
                && is_swapchain_supported,
            handle: physical_device,
        }
    }

    /// Gets a physical device's queue families
    pub fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let has_present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(physical_device, index as u32, *surface)
                    .expect("Failed to get surface support for physical device")
            };
            if queue_family.queue_count > 0 && has_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    /// Checks if the physical device supports the required extensions
    ///
    /// Returns whether or not all required extensions are supported
    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
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
            &utils::constants::DEVICE_EXTENSIONS
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

        true
    }

    /// Gets the swapchain support info for a surface on a physical device
    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> SwapchainSupportInfo {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, *surface)
                .expect("Failed to query for surface capabilities");

            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, *surface)
                .expect("Failed to query for surface formats");

            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, *surface)
                .expect("Failed to query for surface present modes");

            SwapchainSupportInfo {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    /// Helper constructor that creates a logical device from a physical device
    ///
    /// Returns a handle to the created logical device, and it's queue families
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
    ) -> (ash::Device, QueueFamilyIndices) {
        let indices = Device::find_queue_family(instance, physical_device, surface_loader, surface);

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

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

        let physical_device_features = vk::PhysicalDeviceFeatures::default();

        let required_validation_layers: Vec<*const i8> = utils::constants::VALIDATION_LAYERS
            .iter()
            .map(|layer| layer.as_ptr() as *const i8)
            .collect();

        let device_extension_names_cstring: Vec<CString> = utils::constants::DEVICE_EXTENSIONS
            .iter()
            .map(|&extension| CString::new(extension).unwrap())
            .collect();

        let device_extension_names_ptr: Vec<*const i8> = device_extension_names_cstring
            .iter()
            .map(|t| t.as_ptr())
            .collect();

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(&physical_device_features)
            .enabled_layer_names(&required_validation_layers)
            .enabled_extension_names(&device_extension_names_ptr);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_info, None)
                .expect("Failed to create logical device")
        };

        (device, indices)
    }

    /// Helper constructor that creates a Vulkan command pool
    fn create_command_pool(
        device: &ash::Device,
        queue_family: &QueueFamilyIndices,
    ) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family.graphics_family.unwrap());

        unsafe {
            device
                .create_command_pool(&create_info, None)
                .expect("Failed to create command pool")
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
