use ash::vk;
use std::rc::Rc;

use super::device::{Device, QueueFamilyIndices};
use crate::constants::MAX_FRAMES_IN_FLIGHT;

/// Intermediate struct to store the result of creating a Vulkan swapchain
struct VulkanSwapchain {
    loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

pub struct Swapchain {
    /// Handle to the [`Device`] this [`Swapchain`] belongs to
    device: Rc<Device>,

    /// Manages the underlying Vulkan swapchain
    loader: ash::extensions::khr::Swapchain,
    /// Handle to Vulkan swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSwapchainKHR.html
    pub swapchain: vk::SwapchainKHR,
    /// Color format for all images
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html
    _format: vk::Format,
    /// Size, in pixels, of the swapchain
    pub extent: vk::Extent2D,

    /// Depth stencil images drawn to at the same time as color images
    depth_images: Vec<vk::Image>,
    /// GPU memory associated with the depth image at the same index
    depth_image_memories: Vec<vk::DeviceMemory>,
    /// Image views for each depth image
    depth_image_views: Vec<vk::ImageView>,

    /// Handle to Vulkan render pass being used by the graphics pipeline
    pub render_pass: vk::RenderPass,
    /// All framebuffers being used
    pub framebuffers: Vec<vk::Framebuffer>,

    /// Color images that can be be drawn to and presented
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImage.html
    _images: Vec<vk::Image>,
    /// Handles to Vulkan image views for each image in the swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageView.html
    image_views: Vec<vk::ImageView>,

    /// Semaphores for each image that is available to be drawn to
    image_available_semaphores: Vec<vk::Semaphore>,
    /// Semaphores for each image that is finished rendering, and can be drawn to the window
    render_finished_semaphores: Vec<vk::Semaphore>,
    /// Fences for each image that is currently being worked on
    in_flight_fences: Vec<vk::Fence>,
    /// Fences for each image that is currently in flight
    images_in_flight: Vec<vk::Fence>,
    /// Index of frame being worked on (0 to number of framebuffers)
    current_frame: usize,
}

impl Swapchain {
    /// Creates a new [`Swapchain`]
    ///
    /// Under the hood a new Vulkan swapchain is created as well the framebuffers, images, semaphores, and fences
    /// required to make the [`Swapchain`] work.
    pub fn new(device: Rc<Device>) -> Swapchain {
        let family_indices = Device::find_queue_family(
            &device.instance,
            device.physical_device,
            &device.surface_loader,
            &device.surface,
        );

        let vk_swapchain = Swapchain::create_swapchain(
            &device.instance,
            &device.vk(),
            device.physical_device,
            &device.surface_loader,
            &device.surface,
            &family_indices,
            None,
        );

        let image_views =
            Swapchain::create_image_views(&device.vk(), vk_swapchain.format, &vk_swapchain.images);

        let render_pass = Swapchain::create_render_pass(device.as_ref(), vk_swapchain.format);
        let (depth_images, depth_image_memories, depth_image_views) =
            Swapchain::create_depth_resources(
                device.as_ref(),
                image_views.len(),
                vk_swapchain.extent,
            );
        let framebuffers = Swapchain::create_framebuffers(
            &device.vk(),
            render_pass,
            &image_views,
            &depth_image_views,
            &vk_swapchain.extent,
        );

        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Swapchain::create_sync_objects(&device.vk(), vk_swapchain.images.len());

        Swapchain {
            device,

            loader: vk_swapchain.loader,
            swapchain: vk_swapchain.swapchain,
            _format: vk_swapchain.format,
            extent: vk_swapchain.extent,

            _images: vk_swapchain.images,
            image_views,

            depth_images,
            depth_image_memories,
            depth_image_views,

            render_pass,
            framebuffers,

            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        }
    }

    /// Creates a new [`Swapchain`] re-using the old swapchain.
    ///
    /// See also [`Swapchain::new`]
    pub fn from_old_swapchain(
        device: Rc<Device>,
        previous_swapchain: vk::SwapchainKHR,
    ) -> Swapchain {
        let family_indices = Device::find_queue_family(
            &device.instance,
            device.physical_device,
            &device.surface_loader,
            &device.surface,
        );

        let vk_swapchain = Swapchain::create_swapchain(
            &device.instance,
            &device.vk(),
            device.physical_device,
            &device.surface_loader,
            &device.surface,
            &family_indices,
            Some(previous_swapchain),
        );

        let image_views =
            Swapchain::create_image_views(&device.vk(), vk_swapchain.format, &vk_swapchain.images);

        let render_pass = Swapchain::create_render_pass(device.as_ref(), vk_swapchain.format);
        let (depth_images, depth_image_memories, depth_image_views) =
            Swapchain::create_depth_resources(
                device.as_ref(),
                image_views.len(),
                vk_swapchain.extent,
            );
        let framebuffers = Swapchain::create_framebuffers(
            &device.vk(),
            render_pass,
            &image_views,
            &depth_image_views,
            &vk_swapchain.extent,
        );

        let (
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
        ) = Swapchain::create_sync_objects(&device.vk(), vk_swapchain.images.len());

        Swapchain {
            device,

            loader: vk_swapchain.loader,
            swapchain: vk_swapchain.swapchain,
            _format: vk_swapchain.format,
            extent: vk_swapchain.extent,

            _images: vk_swapchain.images,
            image_views,

            depth_images,
            depth_image_memories,
            depth_image_views,

            render_pass,
            framebuffers,

            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight,
            current_frame: 0,
        }
    }

    /// Creates a new [`VulkanSwapchain`]
    fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
        queue_family: &QueueFamilyIndices,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> VulkanSwapchain {
        let swapchain_support =
            Device::query_swapchain_support(physical_device, surface_loader, surface);
        let surface_format = Swapchain::choose_swapchain_format(&swapchain_support.formats);
        let present_mode =
            Swapchain::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Swapchain::choose_swapchain_extent(&swapchain_support.capabilities);

        // Determine the number of images we want to use
        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            swapchain_support.capabilities.max_image_count
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .old_swapchain(old_swapchain.unwrap_or(vk::SwapchainKHR::null()));

        let loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            loader
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };

        let images = unsafe {
            loader
                .get_swapchain_images(swapchain)
                .expect("Failed te get swapchain images")
        };

        VulkanSwapchain {
            loader,
            swapchain,
            images,
            format: surface_format.format,
            extent,
        }
    }

    /// Chooses the most optimal format for the [`Swapchain`]
    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        available_formats.first().unwrap().clone()
    }

    /// Chooses the most optimal present mode for the [`Swapchain`]
    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }

    /// Chooses the most optimal extent for the [`Swapchain`]
    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: 1280.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: 720.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    /// Creates a Vulkan [`ImageView`][ash::vk::ImageView] for every framebuffer in the [`Swapchain`]
    fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
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

    /// Creates a depth stencil image, image view and memory for every framebuffer in the [`Swapchain`].
    ///
    /// Returns the images, device memory, and image views.
    fn create_depth_resources(
        device: &Device,
        count: usize,
        extent: vk::Extent2D,
    ) -> (Vec<vk::Image>, Vec<vk::DeviceMemory>, Vec<vk::ImageView>) {
        let mut depth_images: Vec<vk::Image> = Vec::new();
        let mut depth_image_memories: Vec<vk::DeviceMemory> = Vec::new();
        let mut depth_image_views: Vec<vk::ImageView> = Vec::new();

        let depth_format = Swapchain::find_depth_format(device);

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
                .format(depth_format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let (depth_image, depth_image_memory) =
                device.create_image(&image_info, vk::MemoryPropertyFlags::DEVICE_LOCAL);

            let image_view_info = vk::ImageViewCreateInfo::builder()
                .image(depth_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(depth_format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            depth_image_views.push(unsafe {
                device
                    .vk()
                    .create_image_view(&image_view_info, None)
                    .expect("Failed to create depth image view")
            });
            depth_images.push(depth_image);
            depth_image_memories.push(depth_image_memory);
        }

        (depth_images, depth_image_memories, depth_image_views)
    }

    /// Creates a new render pass with a color and depth attachment for the [`Swapchain`]
    fn create_render_pass(device: &Device, surface_format: vk::Format) -> vk::RenderPass {
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
            .format(Swapchain::find_depth_format(device))
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
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe {
            device
                .vk()
                .create_render_pass(&render_pass_info, None)
                .expect("Failed to create render pass")
        }
    }

    /// Finds a supported depth format
    fn find_depth_format(device: &Device) -> vk::Format {
        device.find_supported_format(
            &[
                vk::Format::D32_SFLOAT,
                vk::Format::D32_SFLOAT_S8_UINT,
                vk::Format::D24_UNORM_S8_UINT,
            ],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    /// Creates a framebuffer for every image in the [`Swapchain`]
    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        depth_image_views: &Vec<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
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
                    .expect("Failed to create framebuffer")
            });
        }

        framebuffers
    }

    /// Creates semaphores and fences to manage synchronization between the CPU and GPU for every framebuffer
    fn create_sync_objects(
        device: &ash::Device,
        image_count: usize,
    ) -> (
        Vec<vk::Semaphore>,
        Vec<vk::Semaphore>,
        Vec<vk::Fence>,
        Vec<vk::Fence>,
    ) {
        let mut sync_objects = (Vec::new(), Vec::new(), Vec::new(), Vec::new());

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                sync_objects.0.push(
                    device
                        .create_semaphore(&semaphore_info, None)
                        .expect("Failed to create semaphore"),
                );
                sync_objects.1.push(
                    device
                        .create_semaphore(&semaphore_info, None)
                        .expect("Failed to create semaphore"),
                );
                sync_objects.2.push(
                    device
                        .create_fence(&fence_info, None)
                        .expect("Failed to create semaphore"),
                );
            };
        }

        for _ in 0..image_count {
            sync_objects.3.push(vk::Fence::null());
        }

        sync_objects
    }
}

impl Swapchain {
    /// Returns the aspect ratio of the [`Swapchain`] extent
    pub fn extent_aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }

    /// Acquires the next available framebuffer that can be drawn to
    ///
    /// Returns the index of the framebuffer that was acquired, and whether the [`Swapchain`] is suboptimal for the surface
    pub fn acquire_next_image(&self) -> Result<(u32, bool), vk::Result> {
        // Wait for previous frame to finish drawing (blocking wait)
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            self.device
                .vk()
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for fences");

            self.loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        }
    }

    /// Submits a draw command buffer to the framebuffer at index, and presents it to the surface
    ///
    /// Returns whether the [`Swapchain`] is suboptimal for the surface, and a vk::Result which contains an Vulkan specific error
    pub fn submit_command_buffers(
        &mut self,
        command_buffer: vk::CommandBuffer,
        index: usize,
    ) -> Result<bool, vk::Result> {
        // Wait for previous image to finish getting drawn
        if vk::Handle::as_raw(self.images_in_flight[index]) != 0 {
            let wait_fences = [self.images_in_flight[index]];
            unsafe {
                self.device
                    .vk()
                    .wait_for_fences(&wait_fences, true, std::u64::MAX)
                    .expect("Failed to wait for fences");
            };
        }
        self.images_in_flight[index] = self.in_flight_fences[self.current_frame];

        // Get GPU to start working on the next frame buffer
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let command_buffers = [command_buffer];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        let reset_fences = [self.in_flight_fences[self.current_frame]];
        unsafe {
            self.device
                .vk()
                .reset_fences(&reset_fences)
                .expect("Failed to reset fences");

            self.device
                .vk()
                .queue_submit(
                    self.device.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit draw command buffer")
        };

        // Present the frame that just finished drawing
        let swapchains = [self.swapchain];
        let image_indices = [index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.loader
                .queue_present(self.device.present_queue, &present_info)
        };

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        result
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.image_views.iter() {
                self.device.vk().destroy_image_view(image_view, None);
            }

            self.loader.destroy_swapchain(self.swapchain, None);

            for i in 0..self.depth_images.len() {
                self.device
                    .vk()
                    .destroy_image_view(*self.depth_image_views.get(i).unwrap(), None);
                self.device
                    .vk()
                    .destroy_image(*self.depth_images.get(i).unwrap(), None);
                self.device
                    .vk()
                    .free_memory(*self.depth_image_memories.get(i).unwrap(), None);
            }

            for &framebuffer in self.framebuffers.iter() {
                self.device.vk().destroy_framebuffer(framebuffer, None);
            }

            self.device.vk().destroy_render_pass(self.render_pass, None);

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .vk()
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .vk()
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device
                    .vk()
                    .destroy_fence(self.in_flight_fences[i], None);
            }
        };
    }
}
