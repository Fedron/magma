use ash::{extensions::ext::DebugUtils, vk};
use std::ffi::CString;
use std::ffi::{c_void, CStr};

use crate::VulkanError;

/// Errors that can be throw by the debugger
#[derive(thiserror::Error, Debug)]
pub enum DebuggerError {
    #[error("Failed to create the Vulkan debug messenger")]
    CantCreate(VulkanError),
    #[error("Missing required validation layers")]
    MissingValidationLayers(Vec<String>),
    #[error(transparent)]
    Other(VulkanError),
}

/// Vulkan/LunarG validation layers that can be enabled for debugging
#[derive(Clone, Copy)]
pub enum DebugLayer {
    /// The main Khronos validation layer
    KhronosValidation,
    /// Prints API calls, parameters, and values
    ApiDump,
}

impl Into<CString> for DebugLayer {
    fn into(self) -> CString {
        match self {
            DebugLayer::KhronosValidation => CString::new("VK_LAYER_KHRONOS_validation").unwrap(),
            DebugLayer::ApiDump => CString::new("VK_LAYER_LUNARG_api_dump").unwrap(),
        }
    }
}

impl Into<String> for DebugLayer {
    fn into(self) -> String {
        match self {
            DebugLayer::KhronosValidation => String::from("VK_LAYER_KHRONOS_validation"),
            DebugLayer::ApiDump => String::from("VK_LAYER_LUNARG_api_dump"),
        }
    }
}

/// Wraps Vulkan debug utils
pub struct Debugger {
    /// Vulkan debug utils extension used to create the messenger
    debug_utils: DebugUtils,
    /// Opaque handle to Vulkan debug utils messenger
    handle: vk::DebugUtilsMessengerEXT,
}

impl Debugger {
    /// Creates a new Vulkan debug messenger that logs performance and validation messages
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        layers: &[DebugLayer],
    ) -> Result<Debugger, DebuggerError> {
        Debugger::check_validation_layers(entry, layers)?;

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
    /// Checks wether the loaded Vulkan library supports the required validation layers
    pub fn check_validation_layers(
        entry: &ash::Entry,
        required_layers: &[DebugLayer],
    ) -> Result<(), DebuggerError> {
        let supported_layers = entry
            .enumerate_instance_layer_properties()
            .map_err(|err| DebuggerError::Other(err.into()))?;

        let is_missing_layers = crate::utils::contains_required(
            &supported_layers
                .iter()
                .map(|layer| crate::utils::char_array_to_string(&layer.layer_name))
                .collect::<Vec<String>>(),
            &required_layers
                .iter()
                .map(|&layer| Into::<String>::into(layer))
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

/// Vulkan callback to print vulkan debug messages using the `log` crate
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
