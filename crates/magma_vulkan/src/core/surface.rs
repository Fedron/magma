use ash::vk;

use crate::{
    core::device::{DeviceExtension, PhysicalDevice, Queue},
    core::instance::Instance,
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum SurfaceError {
    #[error("Can't create a surface on the device provided as it doesn't have the DeviceExtension::Swapchain")]
    DeviceNotCapable,
    #[error("Can't create a surface on a device that wasn't created with a '{0}' queue family")]
    MissingQueueFamily(Queue),
    #[error("Failed to create a surface for Windows")]
    CantCreateWin32Surface(VulkanError),
    #[error("Failed to create a surface for Linux: {0}")]
    CantCreateXlibSurface(VulkanError),
    #[error("Failed to query the surface for properties")]
    FailedQuery(SurfaceQueryType),
}

#[derive(Debug)]
pub enum SurfaceQueryType {
    Capabilities,
    Format,
    PresentModes,
    SurfaceSupport,
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
        if !physical_device
            .enabled_extensions()
            .contains(&DeviceExtension::Swapchain)
        {
            return Err(SurfaceError::DeviceNotCapable);
        }

        let graphics_family = physical_device
            .queue_families()
            .iter()
            .find(|family| family.ty == Queue::Graphics);
        if graphics_family.is_none() {
            return Err(SurfaceError::MissingQueueFamily(Queue::Graphics));
        }

        let surface = ash::extensions::khr::Surface::new(instance.entry(), instance.vk_handle());
        let handle =
            unsafe { Surface::create_surface(instance.entry(), instance.vk_handle(), window)? };

        let supported = if let Some(family) = graphics_family {
            unsafe {
                surface
                    .get_physical_device_surface_support(
                        physical_device.vk_handle(),
                        family.index.unwrap(),
                        handle,
                    )
                    .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::SurfaceSupport))?
            }
        } else {
            false
        };
        if !supported {
            return Err(SurfaceError::DeviceNotCapable);
        }

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

        let hwnd = window.hwnd();
        let hinstance = GetModuleHandleW(std::ptr::null()) as *const c_void;
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd as *const c_void);

        let surface = Win32Surface::new(entry, instance);
        surface
            .create_win32_surface(&create_info, None)
            .map_err(|err| SurfaceError::CantCreateWin32Surface(err.into()))
    }

    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    unsafe fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> Result<vk::SurfaceKHR, SurfaceError> {
        use ash::extensions::khr::XlibSurface;
        use winit::platform::unix::WindowExtUnix;

        let x11_display = window.xlib_display().unwrap();
        let x11_window = window.xlib_window().unwrap();
        let create_info = vk::XlibSurfaceCreateInfoKHR::builder()
            .window(x11_window as vk::Window)
            .dpy(x11_display as *mut vk::Display);

        let surface = XlibSurface::new(entry, instance);
        surface
            .create_xlib_surface(&create_info, None)
            .map_err(|err| SurfaceError::CantCreateXlibSurface(err.into()))
    }
}

impl Surface {
    pub(crate) fn vk_handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

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

impl Surface {
    pub fn update(&mut self, physical_device: &PhysicalDevice) -> Result<(), SurfaceError> {
        let capabilities = unsafe {
            self.surface
                .get_physical_device_surface_capabilities(physical_device.vk_handle(), self.handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::Capabilities))?
        };

        let formats = unsafe {
            self.surface
                .get_physical_device_surface_formats(physical_device.vk_handle(), self.handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::Format))?
        };

        let present_modes = unsafe {
            self.surface
                .get_physical_device_surface_present_modes(physical_device.vk_handle(), self.handle)
                .map_err(|_| SurfaceError::FailedQuery(SurfaceQueryType::PresentModes))?
        };

        self.capabilities = capabilities;
        self.formats = formats;
        self.present_modes = present_modes;

        Ok(())
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.handle, None);
        };
    }
}
