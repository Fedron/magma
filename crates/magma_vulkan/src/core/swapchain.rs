use ash::vk;
use std::{mem::ManuallyDrop, rc::Rc};

use crate::{
    core::{
        commands::buffer::CommandBuffer,
        device::{DeviceExtension, LogicalDevice, LogicalDeviceError, Queue},
        surface::Surface,
    },
    sync::{Fence, Semaphore},
    VulkanError,
};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

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
    #[error("Failed to create a Vulkan render pass: {0}")]
    CantCreateRenderPass(VulkanError),
    #[error("Failed to create a Vulkan framebuffer: {0}")]
    CantCreateFramebuffer(VulkanError),
    #[error("Failed to create a Vulkan image view: {0}")]
    CantCreateImageView(VulkanError),
    #[error("The swapchain is suboptimal for the surface, can still draw but should be recreated")]
    Suboptimal,
    #[error("Can't perform an operation because the graphics queue is required but the device doesn't have one")]
    DeviceMissingGraphicsQueue,
    #[error(transparent)]
    DeviceError(#[from] LogicalDeviceError),
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
    old_swapchain: Option<ManuallyDrop<Swapchain>>,
}

impl SwapchainBuilder {
    pub fn new() -> SwapchainBuilder {
        SwapchainBuilder {
            preferred_color_format: ColorFormat::Unorm,
            preferred_present_mode: PresentMode::Fifo,
            old_swapchain: None,
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

    pub fn old_swapchain(mut self, swapchain: Swapchain) -> SwapchainBuilder {
        self.old_swapchain = Some(ManuallyDrop::new(swapchain));
        self
    }

    pub fn build(
        self,
        device: Rc<LogicalDevice>,
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
        let extent = self.choose_extent(surface.capabilities());

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

        let old_swapchain = if self.old_swapchain.is_some() {
            self.old_swapchain.as_ref().unwrap().handle
        } else {
            vk::SwapchainKHR::null()
        };
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
            .image_array_layers(1)
            .old_swapchain(old_swapchain);

        let swapchain =
            ash::extensions::khr::Swapchain::new(device.instance().vk_handle(), device.vk_handle());
        let handle = unsafe {
            swapchain
                .create_swapchain(&create_info, None)
                .map_err(|err| SwapchainError::CantCreate(err.into()))?
        };

        if self.old_swapchain.is_some() {
            unsafe {
                ManuallyDrop::drop(&mut self.old_swapchain.unwrap());
            };
        }

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

        let depth_format = device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;
        let render_pass = SwapchainBuilder::create_render_pass(
            device.vk_handle(),
            surface_format.format,
            depth_format,
        )?;

        let (depth_images, depth_image_memories, depth_image_views) =
            SwapchainBuilder::create_depth_resources(
                device.as_ref(),
                &depth_format,
                image_views.len(),
                &extent,
            )?;
        let framebuffers = SwapchainBuilder::create_framebuffers(
            device.vk_handle(),
            render_pass,
            &image_views,
            &depth_image_views,
            &extent,
        )?;

        let mut image_available_semaphores: Vec<Semaphore> = Vec::new();
        let mut render_finished_semaphores: Vec<Semaphore> = Vec::new();
        let mut in_flight_fences: Vec<Fence> = Vec::new();
        let mut images_in_flight: Vec<vk::Fence> = Vec::new();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(Semaphore::new(device.clone())?);
            render_finished_semaphores.push(Semaphore::new(device.clone())?);
            in_flight_fences.push(Fence::new(device.clone())?);
        }

        for _ in 0..images.len() {
            images_in_flight.push(vk::Fence::null());
        }

        Ok(Swapchain {
            _images: images,
            image_views,
            depth_images,
            depth_image_views,
            depth_image_memories,

            _format: surface_format.format,
            _depth_format: depth_format,
            extent,

            render_pass,
            framebuffers,

            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,

            swapchain,
            handle,
            device,
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

    fn choose_extent(&self, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
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

    fn create_render_pass(
        device: &ash::Device,
        surface_format: vk::Format,
        depth_format: vk::Format,
    ) -> Result<vk::RenderPass, SwapchainError> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = [vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)
            .depth_stencil_attachment(&depth_attachment_ref)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let render_pass_attachments = [color_attachment, depth_attachment];
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe {
            device
                .create_render_pass(&create_info, None)
                .map_err(|err| SwapchainError::CantCreateRenderPass(err.into()))
        }
    }

    fn create_depth_resources(
        device: &LogicalDevice,
        depth_format: &vk::Format,
        count: usize,
        extent: &vk::Extent2D,
    ) -> Result<(Vec<vk::Image>, Vec<vk::DeviceMemory>, Vec<vk::ImageView>), SwapchainError> {
        let mut depth_images: Vec<vk::Image> = Vec::new();
        let mut depth_image_memories: Vec<vk::DeviceMemory> = Vec::new();
        let mut depth_image_views: Vec<vk::ImageView> = Vec::new();

        for _ in 0..count {
            let image_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(*depth_format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let (depth_image, depth_image_memory) =
                device.create_image(&image_info, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

            let image_view_info = vk::ImageViewCreateInfo::builder()
                .image(depth_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(*depth_format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            depth_image_views.push(unsafe {
                device
                    .vk_handle()
                    .create_image_view(&image_view_info, None)
                    .map_err(|err| SwapchainError::CantCreateImageView(err.into()))?
            });
            depth_images.push(depth_image);
            depth_image_memories.push(depth_image_memory);
        }

        Ok((depth_images, depth_image_memories, depth_image_views))
    }

    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &[vk::ImageView],
        depth_image_views: &[vk::ImageView],
        swapchain_extent: &vk::Extent2D,
    ) -> Result<Vec<vk::Framebuffer>, SwapchainError> {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();
        for (&image_view, &depth_image_view) in image_views.iter().zip(depth_image_views) {
            let attachments = [image_view, depth_image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            framebuffers.push(unsafe {
                device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|err| SwapchainError::CantCreateFramebuffer(err.into()))?
            });
        }

        Ok(framebuffers)
    }
}

pub struct Swapchain {
    _images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    depth_images: Vec<vk::Image>,
    depth_image_views: Vec<vk::ImageView>,
    depth_image_memories: Vec<vk::DeviceMemory>,

    _format: vk::Format,
    _depth_format: vk::Format,
    extent: vk::Extent2D,

    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    image_available_semaphores: Vec<Semaphore>,
    render_finished_semaphores: Vec<Semaphore>,
    in_flight_fences: Vec<Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,

    swapchain: ash::extensions::khr::Swapchain,
    handle: vk::SwapchainKHR,
    device: Rc<LogicalDevice>,
}

impl Swapchain {
    pub fn builder() -> SwapchainBuilder {
        SwapchainBuilder::new()
    }
}

impl Swapchain {
    pub fn extent(&self) -> (u32, u32) {
        (self.extent.width, self.extent.height)
    }

    pub fn render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn framebuffers(&self) -> &[vk::Framebuffer] {
        &self.framebuffers
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }
}

impl Swapchain {
    pub fn acquire_next_image(&self) -> Result<usize, SwapchainError> {
        self.device.wait_for_fences(
            &[&self.in_flight_fences[self.current_frame]],
            true,
            std::u64::MAX,
        )?;

        let result = unsafe {
            self.swapchain
                .acquire_next_image(
                    self.handle,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame].vk_handle(),
                    vk::Fence::null(),
                )
                .map_err(|err| SwapchainError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        if result.1 {
            Err(SwapchainError::Suboptimal)
        } else {
            Ok(result.0 as usize)
        }
    }

    pub fn submit_command_buffer(
        &mut self,
        command_buffer: &CommandBuffer,
        index: usize,
    ) -> Result<(), SwapchainError> {
        // Wait for previous image to finish getting drawn
        if vk::Handle::as_raw(self.images_in_flight[index]) != 0 {
            let wait_fences = [self.images_in_flight[index]];
            unsafe {
                self.device
                    .vk_handle()
                    .wait_for_fences(&wait_fences, true, std::u64::MAX)
                    .map_err(|err| {
                        SwapchainError::DeviceError(LogicalDeviceError::Other(err.into()))
                    })?;
            };
        }
        self.images_in_flight[index] = self.in_flight_fences[self.current_frame].vk_handle();

        // Get GPU to start working on the next frame buffer
        let wait_semaphores = [self.image_available_semaphores[self.current_frame].vk_handle()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame].vk_handle()];

        let command_buffers = [command_buffer.vk_handle()];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        let reset_fences = [&self.in_flight_fences[self.current_frame]];
        self.device.reset_fences(&reset_fences)?;

        let graphics_queue = self.device.queue(Queue::Graphics);
        if graphics_queue.is_none() {
            return Err(SwapchainError::DeviceMissingGraphicsQueue);
        }
        let graphics_queue = graphics_queue.unwrap();

        unsafe {
            self.device
                .vk_handle()
                .queue_submit(
                    graphics_queue.handle,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame].vk_handle(),
                )
                .map_err(|err| SwapchainError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        // Present the frame that just finished drawing
        let swapchains = [self.handle];
        let image_indices = [index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain
                .queue_present(graphics_queue.handle, &present_info)
                .map_err(|err| SwapchainError::DeviceError(LogicalDeviceError::Other(err.into())))?
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        for &image_view in self.image_views.iter() {
            unsafe {
                self.device.vk_handle().destroy_image_view(image_view, None);
            };
        }

        for i in 0..self.depth_images.len() {
            unsafe {
                self.device
                    .vk_handle()
                    .destroy_image_view(*self.depth_image_views.get(i).unwrap(), None);
                self.device
                    .vk_handle()
                    .destroy_image(*self.depth_images.get(i).unwrap(), None);
                self.device
                    .vk_handle()
                    .free_memory(*self.depth_image_memories.get(i).unwrap(), None);
            }
        }

        for &framebuffer in self.framebuffers.iter() {
            unsafe {
                self.device
                    .vk_handle()
                    .destroy_framebuffer(framebuffer, None);
            }
        }

        unsafe {
            self.device
                .vk_handle()
                .destroy_render_pass(self.render_pass, None);
            self.swapchain.destroy_swapchain(self.handle, None);
        };
    }
}
