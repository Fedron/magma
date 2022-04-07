extern crate log;

mod core;
mod pipeline;
mod utils;

use ash::vk::Result as VkResult;

#[derive(thiserror::Error, Debug)]
pub enum VulkanError {
    #[error("Host memory allocation has failed")]
    OutOfHostMemory,
    #[error("Device memory allocation has failed")]
    OutOfDeviceMemory,
    #[error("Initialization of an object could not be completed")]
    InitializationFailed,
    #[error("The logical or physical device was lost")]
    DeviceLost,
    #[error("Mapping of memory failed")]
    MemoryMapFailed,
    #[error("A requested layer is not present or could not be loaded")]
    LayerNotPresent,
    #[error("A requested extension is not supported")]
    ExtensionNotPresent,
    #[error("A requested feature is not supported")]
    FeatureNotPresent,
    #[error("The requested version of Vulkan is not supported by the driver, or is otherwise incompatible")]
    IncompatibleDriver,
    #[error("Too many objects of the type have already been created")]
    TooManyObjects,
    #[error("A requested format is not supported on this device")]
    FormatNotSupported,
    #[error("A pool allocation has failed due to fragmentation of the pool's memory")]
    FragmentedPool,
    #[error("A surface is no longer available")]
    SurfaceLost,
    #[error("The requested window is already in use by Vulkan or another API")]
    NativeWindowInUse,
    #[error("A surface has changed in a way that it is no longer compatible with the swapchain")]
    OutOfDateSurface,
    #[error("The display used by the swapchain does not use the same presentable image layout, or is incompatible in a way that prevents sharing an image")]
    IncompatibleDisplay,
    #[error("One or more shaders failed to compile or link")]
    InvalidShader,
    #[error("A pool memory allocation has failed")]
    OutOfPoolMemory,
    #[error("An external handle is not a valid handle of the specified type")]
    InvalidExternalHandle,
    #[error("A descriptor pool creation has failed due to fragmentation")]
    Fragmentation,
    #[error("A buffer creation failed because the requested address is not available")]
    InvalidDeviceAddress,
    #[error("A buffer creation or memory allocation failed because the requested address is not available")]
    InvalidCaptureAddress,
    #[error("An operation on a swapchain created with exclusive full-screen access failed as it did not have exclusive full-screen access")]
    LostFullscreenAccess,
    #[error("An unknown error has occurred")]
    Unknown,
    #[error("Other Vulkan error")]
    Other(VkResult),
}

impl From<VkResult> for VulkanError {
    fn from(e: VkResult) -> Self {
        if e == VkResult::ERROR_OUT_OF_HOST_MEMORY {
            VulkanError::OutOfHostMemory
        } else if e == VkResult::ERROR_OUT_OF_DEVICE_MEMORY {
            VulkanError::OutOfDeviceMemory
        } else if e == VkResult::ERROR_INITIALIZATION_FAILED {
            VulkanError::InitializationFailed
        } else if e == VkResult::ERROR_DEVICE_LOST {
            VulkanError::DeviceLost
        } else if e == VkResult::ERROR_MEMORY_MAP_FAILED {
            VulkanError::MemoryMapFailed
        } else if e == VkResult::ERROR_LAYER_NOT_PRESENT {
            VulkanError::LayerNotPresent
        } else if e == VkResult::ERROR_EXTENSION_NOT_PRESENT {
            VulkanError::ExtensionNotPresent
        } else if e == VkResult::ERROR_FEATURE_NOT_PRESENT {
            VulkanError::FeatureNotPresent
        } else if e == VkResult::ERROR_INCOMPATIBLE_DRIVER {
            VulkanError::IncompatibleDriver
        } else if e == VkResult::ERROR_TOO_MANY_OBJECTS {
            VulkanError::TooManyObjects
        } else if e == VkResult::ERROR_FORMAT_NOT_SUPPORTED {
            VulkanError::FormatNotSupported
        } else if e == VkResult::ERROR_FRAGMENTED_POOL {
            VulkanError::FragmentedPool
        } else if e == VkResult::ERROR_SURFACE_LOST_KHR {
            VulkanError::SurfaceLost
        } else if e == VkResult::ERROR_NATIVE_WINDOW_IN_USE_KHR {
            VulkanError::NativeWindowInUse
        } else if e == VkResult::ERROR_OUT_OF_DATE_KHR {
            VulkanError::OutOfDateSurface
        } else if e == VkResult::ERROR_INCOMPATIBLE_DISPLAY_KHR {
            VulkanError::IncompatibleDisplay
        } else if e == VkResult::ERROR_INVALID_SHADER_NV {
            VulkanError::InvalidShader
        } else if e == VkResult::ERROR_OUT_OF_POOL_MEMORY
            || e == VkResult::ERROR_OUT_OF_POOL_MEMORY_KHR
        {
            VulkanError::OutOfPoolMemory
        } else if e == VkResult::ERROR_INVALID_EXTERNAL_HANDLE
            || e == VkResult::ERROR_INVALID_EXTERNAL_HANDLE_KHR
        {
            VulkanError::InvalidExternalHandle
        } else if e == VkResult::ERROR_FRAGMENTATION || e == VkResult::ERROR_FRAGMENTATION_EXT {
            VulkanError::Fragmentation
        } else if e == VkResult::ERROR_INVALID_DEVICE_ADDRESS_EXT {
            VulkanError::InvalidDeviceAddress
        } else if e == VkResult::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS
            || e == VkResult::ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR
        {
            VulkanError::InvalidCaptureAddress
        } else if e == VkResult::ERROR_FULL_SCREEN_EXCLUSIVE_MODE_LOST_EXT {
            VulkanError::LostFullscreenAccess
        } else if e == VkResult::ERROR_UNKNOWN {
            VulkanError::Unknown
        } else {
            VulkanError::Other(e)
        }
    }
}

pub mod prelude {
    pub use crate::core::commands::buffer::{CommandBuffer, CommandBufferLevel};
    pub use crate::core::commands::pool::{CommandPool, CommandPoolError};
    pub use crate::core::device::{
        DeviceExtension, LogicalDevice, LogicalDeviceError, PhysicalDevice, PhysicalDeviceBuilder,
        PhysicalDeviceError, PhysicalDeviceType, Queue, QueueFamily,
    };
    pub use crate::core::instance::{Instance, InstanceError};
    pub use crate::core::surface::{Surface, SurfaceError};
    pub use crate::core::swapchain::{ColorFormat, PresentMode, Swapchain, SwapchainError};

    pub use crate::pipeline::shader::{ShaderBuilder, ShaderError, ShaderStage};
    pub use crate::pipeline::{Pipeline, PipelineBuilder, PipelineError};
}
