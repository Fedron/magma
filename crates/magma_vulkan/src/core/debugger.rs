use std::ffi::{c_void, CStr};
use ash::{extensions::ext::DebugUtils, vk};

use crate::VulkanError;

pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];

#[derive(thiserror::Error, Debug)]
pub enum DebuggerError {
    #[error("Failed to create the Vulkan debug messenger")]
    CantCreate(VulkanError),
    #[error("Missing required validation layers")]
    MissingValidationLayers(Vec<String>),
    #[error(transparent)]
    Other(VulkanError),
}

pub struct Debugger {
    debug_utils: DebugUtils,
    handle: vk::DebugUtilsMessengerEXT,
}

impl Debugger {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Result<Debugger, DebuggerError> {
        Debugger::check_validation_layers(entry)?;

        let debug_utils = DebugUtils::new(entry, instance);
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
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let handle = unsafe {
            debug_utils
                .create_debug_utils_messenger(&create_info, None)
                .map_err(|err| DebuggerError::CantCreate(err.into()))?
        };

        Ok(Debugger {
            debug_utils,
            handle,
        })
    }
}

impl Debugger {
    pub fn check_validation_layers(entry: &ash::Entry) -> Result<(), DebuggerError> {
        let supported_layers = entry
            .enumerate_instance_layer_properties()
            .map_err(|err| DebuggerError::Other(err.into()))?;
        println!("{:#?}", supported_layers);

        let is_missing_layers = crate::utils::contains_required(
            &supported_layers
                .iter()
                .map(|layer| crate::utils::char_array_to_string(&layer.layer_name))
                .collect::<Vec<String>>(),
            &VALIDATION_LAYERS
                .iter()
                .map(|&layer| layer.to_string())
                .collect::<Vec<String>>(),
        );

        if is_missing_layers.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_layers.1
            );
            Err(DebuggerError::MissingValidationLayers(is_missing_layers.1))
        } else {
            Ok(())
        }
    }
}

impl Drop for Debugger {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.handle, None);
        };
    }
}

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
