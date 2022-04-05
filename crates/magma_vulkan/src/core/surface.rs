use ash::vk;

use crate::{core::instance::Instance, VulkanError};

#[derive(thiserror::Error, Debug)]
pub enum SurfaceError {
    #[error("Failed to create a surface for Windows")]
    CantCreateWin32Surface(VulkanError),
}

pub struct Surface {
    surface: ash::extensions::khr::Surface,
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        instance: &Instance,
        window: &winit::window::Window,
    ) -> Result<Surface, SurfaceError> {
        let surface = ash::extensions::khr::Surface::new(instance.entry(), instance.vk_handle());
        let handle =
            unsafe { Surface::create_surface(instance.entry(), instance.vk_handle(), window)? };

        Ok(Surface { surface, handle })
    }

    #[cfg(target_os = "windows")]
    unsafe fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use ash::extensions::khr::Win32Surface;
        use std::os::raw::c_void;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd() as *const c_void;
        let hinstance = GetModuleHandleW(std::ptr::null()) as *const c_void;
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd);

        let surface = Win32Surface::new(entry, instance);
        surface
            .create_win32_surface(&create_info, None)
            .map_err(|err| SurfaceError::CantCreateWin32Surface(err.into()))
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.handle, None);
        };
    }
}
