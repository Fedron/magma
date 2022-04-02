use std::ffi::CString;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    vk, Entry,
};

use crate::{
    debugger::{Debugger, ENABLE_VALIDATION_LAYERS},
    utils,
};

pub struct Instance {
    handle: ash::Instance,
    entry: Entry,
}

impl Instance {
    pub fn new() -> Instance {
        let entry = unsafe { ash::Entry::load().expect("Failed to lead Vulkan library") };

        // Create vulkan instance
        let required_extension_names = required_extension_names();
        Instance::check_required_extensions(&entry, &required_extension_names);

        let app_name = CString::new("Magma App").unwrap();
        let engine_name = CString::new("Magma").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name)
            .api_version(vk::make_api_version(0, 1, 2, 0));

        let enabled_layer_names = if ENABLE_VALIDATION_LAYERS {
            Debugger::validation_layers()
        } else {
            Vec::new()
        };

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&required_extension_names)
            .enabled_layer_names(&enabled_layer_names);

        let handle = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance")
        };

        Instance { entry, handle }
    }

    fn check_required_extensions(entry: &Entry, required_extension_names: &[*const i8]) {
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
    }
}

impl Instance {
    pub fn vk_handle(&self) -> &ash::Instance {
        &self.handle
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}

#[cfg(all(windows))]
fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}
