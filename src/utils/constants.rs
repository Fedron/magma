pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];
pub const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
