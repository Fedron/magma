#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::vk;

use crate::{utils, VulkanError};

#[derive(thiserror::Error, Debug)]
pub enum InstanceError {
    #[error(transparent)]
    LoadLibraryError(#[from] ash::LoadingError),
    #[error("Creating instance failed")]
    CreateError(VulkanError),
    #[error("Missing required extensions")]
    MissingExtensions(Vec<String>),
    #[error(transparent)]
    Other(VulkanError),
}

pub struct Instance {
    handle: ash::Instance,
    entry: ash::Entry,
}

impl Instance {
    pub fn new() -> Result<Instance, InstanceError> {
        let entry =
            unsafe { ash::Entry::load().map_err(|err| InstanceError::LoadLibraryError(err))? };

        Instance::check_required_extensions(&entry)?;

        use std::ffi::CString;
        let app_name = CString::new("Magma").unwrap();
        let engine_name = CString::new("Magma").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name);

        let enabled_extension_names = Instance::required_extension_names();
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names);

        let handle = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|err| InstanceError::CreateError(err.into()))?
        };

        Ok(Instance { entry, handle })
    }

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

    #[cfg(all(windows))]
    fn required_extension_names() -> Vec<*const i8> {
        vec![
            Surface::name().as_ptr(),
            Win32Surface::name().as_ptr(),
            DebugUtils::name().as_ptr(),
        ]
    }
}

impl Instance {
    pub(crate) fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub(crate) fn vk_handle(&self) -> &ash::Instance {
        &self.handle
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}
