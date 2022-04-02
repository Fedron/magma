use ash::{extensions::ext::DebugUtils, vk};
use std::ffi::{c_void, CStr};

use crate::instance::Instance;

pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub struct Debugger {
    handle: vk::DebugUtilsMessengerEXT,
    debug_utils: DebugUtils,
}

impl Debugger {
    pub fn new(instance: &Instance) -> Debugger {
        Debugger::check_validation_layer_support(instance.entry());

        let debug_utils = DebugUtils::new(instance.entry(), instance.vk_handle());
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(_vulkan_debug_utils_callback));

        let handle = unsafe {
            debug_utils
                .create_debug_utils_messenger(&create_info, None)
                .expect("Failed to create Vulkan debug messenger")
        };

        Debugger {
            handle,
            debug_utils,
        }
    }
}

impl Debugger {
    pub fn check_validation_layer_support(entry: &ash::Entry) {
        let supported_layers = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to get instance layer properties");

        let is_missing_layers = crate::utils::contains_required(
            &supported_layers
                .iter()
                .map(|layer| crate::utils::char_array_to_string(&layer.layer_name))
                .collect::<Vec<String>>(),
            &VALIDATION_LAYERS
                .iter()
                .map(|&layer| String::from(layer))
                .collect::<Vec<String>>(),
        );

        if is_missing_layers.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_layers.1
            );
            panic!("Missing extensions, see above")
        }
    }
}

unsafe extern "system" fn _vulkan_debug_utils_callback(
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
            log::debug!("[Vulkan] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("[Vulkan] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("[Vulkan] {} {:?}", type_, message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("[Vulkan] {} {:?}", type_, message)
        }
        _ => {}
    };

    vk::FALSE
}
