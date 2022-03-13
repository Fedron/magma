use ash::vk;
use std::{
    collections::HashSet,
    ffi::{CStr, CString},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::utils;

const WINDOW_TITLE: &'static str = "Magma";

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

        App {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
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
        let required_hash_set = HashSet::<&CStr>::from_iter(
            required_extension_names
                .iter()
                .map(|&extension| unsafe { CStr::from_ptr(extension) })
                .collect::<Vec<&CStr>>(),
        );
        let supported_hash_set = &HashSet::<&CStr>::from_iter(
            supported_extension_names
                .iter()
                .map(|extension| unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) })
                .collect::<Vec<&CStr>>(),
        );
        let missing_extensions = required_hash_set
            .difference(supported_hash_set)
            .collect::<Vec<&&CStr>>();

        if missing_extensions.len() > 0 {
            log::error!(
                "Your device is missing required features: {:?}",
                missing_extensions
            );
            return false;
        }

        true
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
            if utils::constants::ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        };
    }
}
