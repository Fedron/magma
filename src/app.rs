use ash::vk;
use std::{collections::HashSet, ffi::CString};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::utils;

const WINDOW_TITLE: &'static str = "Magma";

/// Wrapper struct with all the queue families required for the app
struct QueueFamilyIndices {
    /// Index of the graphics queue family
    graphics_family: Option<u32>,
}

/// Contains information about a Vulkan physical device, as well as a handle to the device
struct PhysicalDeviceInfo<'a> {
    name: String,
    _device_id: u32,
    device_type: &'a str,
    is_suitable: bool,
    handle: vk::PhysicalDevice,
}

impl std::fmt::Display for PhysicalDeviceInfo<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.device_type)
    }
}

impl QueueFamilyIndices {
    /// Returns whether or not all the queue family indices are present
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

pub struct App {
    /// Holds the loaded Vulkan library
    _entry: ash::Entry,
    /// Handle to the Vulkan instance
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkInstance.html
    instance: ash::Instance,
    /// Manages the debug_messenger
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    /// Handle to Vulkan debug messenger
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDebugUtilsMessengerEXT.html
    debug_messenger: vk::DebugUtilsMessengerEXT,
    /// Handle to Vulkan physical device this app is using
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html
    _physical_device: vk::PhysicalDevice,
    /// Handle to Vulkan logical device
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDevice.html
    device: ash::Device,
    /// Handle to Vulkan queue used for graphics operations
    _graphics_queue: vk::Queue,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new() -> App {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan library") };
        let instance = App::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) =
            utils::debug::setup_debug_utils(&entry, &instance);
        let physical_device = App::pick_physical_device(&instance);
        let (logical_device, graphics_queue) =
            App::create_logical_device(&instance, physical_device);

        App {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            _physical_device: physical_device,
            device: logical_device,
            _graphics_queue: graphics_queue,
        }
    }

    /// Constructor to create a Vulkan instance
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkInstance.html
    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        let required_extension_names = utils::platforms::required_extension_names();
        if !App::check_required_extensions(entry, &required_extension_names) {
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
            Vec::new()
        } else {
            utils::constants::VALIDATION_LAYERS
                .iter()
                .map(|layer| layer.as_ptr() as *const i8)
                .collect::<Vec<*const i8>>()
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
        let required_hash_set = HashSet::<String>::from_iter(
            required_extension_names
                .iter()
                .map(|&extension| utils::char_ptr_to_string(extension))
                .collect::<Vec<String>>(),
        );
        let supported_hash_set = &HashSet::<String>::from_iter(
            supported_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
        );
        let missing_extensions = required_hash_set
            .difference(supported_hash_set)
            .collect::<Vec<&String>>();

        if missing_extensions.len() > 0 {
            log::error!(
                "Your device is missing required features: {:?}",
                missing_extensions
            );
            return false;
        }

        true
    }

    /// Finds a Vulkan physical device that matches the needs of the application, and returns it
    fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Vulkan physical devices")
        };

        let mut chosen_device: Option<PhysicalDeviceInfo> = None;
        for &physical_device in physical_devices.iter() {
            let physical_device_info = App::is_physical_device_suitable(instance, physical_device);
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
    ) -> PhysicalDeviceInfo {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let _device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            _ => "Unknown",
        };

        let indices = App::find_queue_family(instance, physical_device);

        PhysicalDeviceInfo {
            name: utils::char_array_to_string(&device_properties.device_name),
            _device_id: device_properties.device_id,
            device_type,
            is_suitable: indices.is_complete(),
            handle: physical_device,
        }
    }

    /// Gets a physical device's queue families
    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    /// Creates a Vulkan logical device from a physical device
    ///
    /// Returns a handle to the created logical device, and it's graphics queue
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> (ash::Device, vk::Queue) {
        let indices = App::find_queue_family(instance, physical_device);

        let queue_priorities = [1.0_f32];
        let queue_infos = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .queue_priorities(&queue_priorities)
            .build()];

        let physical_device_features = vk::PhysicalDeviceFeatures::default();

        let required_validation_layers: Vec<*const i8> = utils::constants::VALIDATION_LAYERS
            .iter()
            .map(|layer| layer.as_ptr() as *const i8)
            .collect();

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(&physical_device_features)
            .enabled_layer_names(&required_validation_layers);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_info, None)
                .expect("Failed to create logical device")
        };

        let graphics_queue =
            unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };

        (device, graphics_queue)
    }

    /// Initialises a winit window, returning the initialised window
    pub fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                utils::constants::WINDOW_WIDTH,
                utils::constants::WINDOW_HEIGHT,
            ))
            .build(event_loop)
            .expect("")
    }

    pub fn draw_frame(&mut self) {}

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => self.draw_frame(),
            _ => {}
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);

            if utils::constants::ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
        };
    }
}
