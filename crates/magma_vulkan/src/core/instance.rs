use std::mem::ManuallyDrop;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::vk;

use super::debugger::{DebugLayer, Debugger, DebuggerError};
use crate::{
    utils, VulkanError,
};

/// Errors that can be thrown by the Vulkan instance
#[derive(thiserror::Error, Debug)]
pub enum InstanceError {
    #[error(transparent)]
    LoadLibraryError(#[from] ash::LoadingError),
    #[error("Creating instance failed: {0}")]
    CantCreate(VulkanError),
    #[error("Missing required extensions")]
    MissingExtensions(Vec<String>),
    #[error(transparent)]
    CantCreateDebugger(#[from] DebuggerError),
    #[error(transparent)]
    Other(VulkanError),
}

/// Wraps a Vulkan instance and loaded library
pub struct Instance {
    /// List of Vulkan validation layers used by the [Debugger]
    debug_layers: Vec<DebugLayer>,
    /// Handle to the created debugger
    debugger: ManuallyDrop<Option<Debugger>>,
    /// Opaque handle to Vulkan instance
    handle: ash::Instance,
    /// Opaque handle to loaded Vulkan library
    entry: ash::Entry,
}

impl Instance {
    /// Creates a new instance that loads the Vulkan library
    ///
    /// Automatically creates a [Debugger] if `magma` is being built in debug mode
    pub fn new(debug_layers: &[DebugLayer]) -> Result<Instance, InstanceError> {
        let entry =
            unsafe { ash::Entry::load().map_err(|err| InstanceError::LoadLibraryError(err))? };

        Instance::check_required_extensions(&entry)?;
        if !debug_layers.is_empty() {
            Debugger::check_validation_layers(&entry, debug_layers)?;
        }

        use std::ffi::CString;
        let app_name = CString::new("Magma").unwrap();
        let engine_name = CString::new("Magma").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name);

        let enabled_extension_names = Instance::required_extension_names();
        let enabled_layer_names_raw: Vec<CString> = debug_layers
            .iter()
            .map(|&layer| Into::<CString>::into(layer))
            .collect();
        let enabled_layer_names: Vec<*const i8> = enabled_layer_names_raw
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_layer_names(&enabled_layer_names);

        let handle = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|err| InstanceError::CantCreate(err.into()))?
        };

        let debugger: Option<Debugger> = if !debug_layers.is_empty() {
            log::debug!("Created Vulkan debugger");
            Some(Debugger::new(&entry, &handle, debug_layers)?)
        } else {
            None
        };

        Ok(Instance {
            debug_layers: debug_layers.to_vec(),
            debugger: ManuallyDrop::new(debugger),
            entry,
            handle,
        })
    }

    /// Checks whether the instance supports all the extensions needed
    ///
    /// See [`Instance::required_extension_names`]
    fn check_required_extensions(entry: &ash::Entry) -> Result<(), InstanceError> {
        let supported_extension_names = entry
            .enumerate_instance_extension_properties(None)
            .map_err(|err| InstanceError::Other(err.into()))?;

        let is_missing_extensions = utils::contains_required(
            &supported_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &Instance::required_extension_names()
                .iter()
                .map(|&extension| utils::char_ptr_to_string(extension))
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            Err(InstanceError::MissingExtensions(is_missing_extensions.1))
        } else {
            Ok(())
        }
    }

    /// Gets the names of all the required extensions on Windows
    #[cfg(all(windows))]
    fn required_extension_names() -> Vec<*const i8> {
        vec![
            Surface::name().as_ptr(),
            Win32Surface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ]
    }

    /// Gets the names of al the required exetensions on Linux
    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    fn required_extension_names() -> Vec<*const i8> {
        vec![
            Surface::name().as_ptr(),
            XlibSurface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ]
    }
}

impl Instance {
    /// Returns the handle to the loaded Vulkan library
    pub(crate) fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    /// Returns the handle to the Vulkan instance
    pub(crate) fn vk_handle(&self) -> &ash::Instance {
        &self.handle
    }

    /// Returns a list of the debug layers used by the debugger
    pub fn debug_layers(&self) -> &[DebugLayer] {
        &self.debug_layers
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.debugger);
            self.handle.destroy_instance(None);
        }
    }
}
