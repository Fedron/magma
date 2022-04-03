use ash::vk;

use crate::prelude::Instance;

pub struct Surface {
    handle: vk::SurfaceKHR,
    surface: ash::extensions::khr::Surface,
}

impl Surface {
    pub fn new(instance: &Instance, window: &winit::window::Window) -> Surface {
        let handle = unsafe {
            Surface::create_vk_surface(instance, window).expect("Failed to create Vulkan surface")
        };
        let surface = ash::extensions::khr::Surface::new(instance.entry(), instance.vk_handle());

        Surface { handle, surface }
    }

    #[cfg(target_os = "windows")]
    unsafe fn create_vk_surface(
        instance: &Instance,
        window: &winit::window::Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        use ash::extensions::khr::Win32Surface;
        use std::os::raw::c_void;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use winit::platform::windows::WindowExtWindows;

        let hwnd = window.hwnd() as *const c_void;
        let hinstance = GetModuleHandleW(std::ptr::null()) as *const c_void;
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd);

        let surface = Win32Surface::new(instance.entry(), instance.vk_handle());
        surface.create_win32_surface(&create_info, None)
    }
}

impl Surface {
    pub fn vk_handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

    pub fn surface(&self) -> &ash::extensions::khr::Surface {
        &self.surface
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface.destroy_surface(self.handle, None);
        };
    }
}
