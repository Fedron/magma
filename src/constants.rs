/// Controls whether the Vulkan debugger should be created
pub const ENABLE_VALIDATION_LAYERS: bool = cfg!(debug_assertions);
/// Validation layers to enable in the Vulkan Debugger
pub const VALIDATION_LAYERS: [&'static str; 1] = ["VK_LAYER_KHRONOS_validation"];
/// Device extensions the [`Renderer`] requires before being able to be created
pub const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];
/// Number of frames to render to 
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;