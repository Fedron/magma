#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;

/// Gets the required extensions te create a Vulkan instance on Windows
#[cfg(all(windows))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}
