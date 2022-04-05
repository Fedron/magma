use ash::vk;

use crate::{
    core::device::PhysicalDevice, core::instance::Instance, prelude::DeviceExtension, VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum SurfaceError {
    #[error("Can't create a surface on the device provided as it doesn't have the DeviceExtension::Swapchain")]
    PhysicalDeviceNotCapable,
    #[error("Failed to create a surface for Windows")]
    CantCreateWin32Surface(VulkanError),
    #[error("Failed to query the surface for properties")]
    FailedQuery(SurfaceQueryType),
}

#[derive(Debug)]
pub enum SurfaceQueryType {
    Capabilities,
    Format,
    PresentModes,
}

pub struct Surface {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,

    surface: ash::extensions::khr::Surface,
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        window: &winit::window::Window,
    ) -> Result<Surface, SurfaceError> {
        if physical_device
            .extensions()
            .iter()
            .find(|&&extension| extension == DeviceExtension::Swapchain)
            .is_none()
        {
            return Err(SurfaceError::PhysicalDeviceNotCapable);
        }

        let surface = ash::extensions::khr::Surface::new(instance.entry(), instance.vk_handle());
        let handle =
            unsafe { Surface::create_surface(instance.entry(), instance.vk_handle(), window)? };

        let capabilities = unsafe {
            surface
                .get_physical_device_surface_capabilities(physical_device.vk_handle(), handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::Capabilities))?
        };

        let formats = unsafe {
            surface
                .get_physical_device_surface_formats(physical_device.vk_handle(), handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::Format))?
        };

        let present_modes = unsafe {
            surface
                .get_physical_device_surface_present_modes(physical_device.vk_handle(), handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::PresentModes))?
        };

        Ok(Surface {
            capabilities,
            formats,
            present_modes,
            surface,
            handle,
        })
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

impl Surface {
    pub fn capabilities(&self) -> &vk::SurfaceCapabilitiesKHR {
        &self.capabilities
    }

    pub fn formats(&self) -> &[vk::SurfaceFormatKHR] {
        &self.formats
    }

    pub fn present_modes(&self) -> &[vk::PresentModeKHR] {
        &self.present_modes
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.handle, None);
        };
    }
}
