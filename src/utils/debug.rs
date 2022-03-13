use ash::{
    extensions::ext::DebugUtils,
    vk::{self, DebugUtilsMessengerEXT},
};
use std::{collections::HashSet, ffi::CStr, os::raw::c_void};

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let type_ = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("[Vulkan::debug] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("[Vulkan::debug] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("[Vulkan::debug] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::error!("[Vulkan::debug] {} {:?}", type_, message)
        }
        _ => {}
    };

    vk::FALSE
}

/// Checks if the Vulkan instance supports the required validation layers
///
/// Returns whether or not all required layers are supported
pub fn check_validation_layer_support(
    entry: &ash::Entry,
    required_validation_layers: &[&str],
) -> bool {
    let supported_layers = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to get instance layer properties");
    let required_hash_set = HashSet::<&CStr>::from_iter(
        required_validation_layers
            .iter()
            .map(|&layer| unsafe { CStr::from_ptr(layer.as_ptr() as *const i8) })
            .collect::<Vec<&CStr>>(),
    );
    let supported_hash_set = &HashSet::<&CStr>::from_iter(
        supported_layers
            .iter()
            .map(|layer| unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) })
            .collect::<Vec<&CStr>>(),
    );
    let missing_layers = required_hash_set
        .difference(supported_hash_set)
        .collect::<Vec<&&CStr>>();

    if missing_layers.len() > 0 {
        log::error!(
            "Your device is missing required layers: {:?}",
            missing_layers
        );
        return false;
    }

    true
}

/// Creates and sets up the Vulkan debug messenger and loader
/// 
/// Returns the debug utils loader and messenger
pub fn setup_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> (DebugUtils, DebugUtilsMessengerEXT) {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback));
    
    let debug_messenger = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .expect("Failed to create debug messenger")
    };

    (debug_utils_loader, debug_messenger)
}
