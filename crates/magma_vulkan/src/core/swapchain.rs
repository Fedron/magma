use ash::vk;

use crate::{
    core::{
        device::{DeviceExtension, LogicalDevice, Queue},
        surface::Surface,
    },
    VulkanError,
};

#[derive(thiserror::Error, Debug)]
pub enum SwapchainError {
    #[error("Failed to create Vulkan swapchain: {0}")]
    CantCreate(VulkanError),
    #[error("Failed to get the images created with the Vulkan swapchain")]
    ImageFetchFail,
    #[error("Can't create a swapchain on the given device since DeviceExtension::Swapchain is not enabled")]
    DeviceNotCapable,
    #[error(
        "Can't create a swapchain because the device wasn't created with a '{0}' queue family"
    )]
    MissingQueueFamily(Queue),
}

#[derive(Clone, Copy)]
pub enum ColorFormat {
    Srgb,
    Unorm,
}

impl Into<vk::Format> for ColorFormat {
    fn into(self) -> vk::Format {
        match self {
            ColorFormat::Srgb => vk::Format::B8G8R8A8_SRGB,
            ColorFormat::Unorm => vk::Format::B8G8R8A8_UNORM,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PresentMode {
    Immediate,
    Mailbox,
    Fifo,
    FifoRelaxed,
}

impl Into<vk::PresentModeKHR> for PresentMode {
    fn into(self) -> vk::PresentModeKHR {
        match self {
            PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
            PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
            PresentMode::Fifo => vk::PresentModeKHR::FIFO,
            PresentMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        }
    }
}

pub struct SwapchainBuilder {
    preferred_color_format: ColorFormat,
    preferred_present_mode: PresentMode,
}

impl SwapchainBuilder {
    pub fn new() -> SwapchainBuilder {
        SwapchainBuilder {
            preferred_color_format: ColorFormat::Unorm,
            preferred_present_mode: PresentMode::Fifo,
        }
    }

    pub fn preferred_color_format(mut self, color_format: ColorFormat) -> SwapchainBuilder {
        self.preferred_color_format = color_format;
        self
    }

    pub fn preferred_present_mode(mut self, present_mode: PresentMode) -> SwapchainBuilder {
        self.preferred_present_mode = present_mode;
        self
    }

    pub fn build(
        self,
        device: &LogicalDevice,
        surface: &Surface,
    ) -> Result<Swapchain, SwapchainError> {
        if !device
            .physical_device()
            .enabled_extensions()
            .contains(&DeviceExtension::Swapchain)
        {
            return Err(SwapchainError::DeviceNotCapable);
        }

        if !device
            .physical_device()
            .queue_families()
            .iter()
            .any(|family| family.ty == Queue::Graphics)
        {
            return Err(SwapchainError::MissingQueueFamily(Queue::Graphics));
        }

        let surface_format = self.choose_format(&surface.formats());
        let present_mode = self.choose_present_mode(&surface.present_modes());
        let extent = SwapchainBuilder::choose_extent(surface.capabilities());

        let image_count = surface.capabilities().min_image_count + 1;
        let image_count = if surface.capabilities().max_image_count > 0
            && image_count > surface.capabilities().max_image_count
        {
            surface.capabilities().max_image_count
        } else {
            image_count
        };

        let queue_family_indices: Vec<u32> = device
            .physical_device()
            .queue_families()
            .iter()
            .map(|family| family.index.unwrap())
            .collect();

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.vk_handle())
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(surface.capabilities().current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain =
            ash::extensions::khr::Swapchain::new(device.instance().vk_handle(), device.vk_handle());
        let handle = unsafe {
            swapchain
                .create_swapchain(&create_info, None)
                .map_err(|err| SwapchainError::CantCreate(err.into()))?
        };

        let images = unsafe {
            swapchain
                .get_swapchain_images(handle)
                .map_err(|_| SwapchainError::ImageFetchFail)?
        };
        let image_views = SwapchainBuilder::create_image_views(
            device.vk_handle(),
            surface_format.format,
            &images,
        );

        Ok(Swapchain {
            images,
            image_views,
            format: surface_format.format,
            extent,

            swapchain,
            handle,
        })
    }
}

impl SwapchainBuilder {
    fn choose_format(&self, available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == self.preferred_color_format.into()
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        log::info!(
            "Preferred color format not supported by device, resorting to {:?}",
            available_formats.first().unwrap().format
        );
        available_formats.first().unwrap().clone()
    }

    fn choose_present_mode(
        &self,
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        if available_present_modes.contains(&self.preferred_present_mode.into()) {
            self.preferred_present_mode.into()
        } else {
            log::info!("Preferred present mode not supported by device, resorting to FIFO");
            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: 800.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: 600.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &[vk::Image],
    ) -> Vec<vk::ImageView> {
        let mut image_views: Vec<vk::ImageView> = Vec::new();
        for &image in images.iter() {
            let create_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);

            image_views.push(unsafe {
                device
                    .create_image_view(&create_info, None)
                    .expect("Failed to create image view")
            });
        }

        image_views
    }
}

pub struct Swapchain {
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    extent: vk::Extent2D,

    swapchain: ash::extensions::khr::Swapchain,
    handle: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn builder() -> SwapchainBuilder {
        SwapchainBuilder::new()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(self.handle, None);
        };
    }
}
