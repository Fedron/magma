use std::rc::Rc;

use ash::vk;

use super::device::{Device, QueueFamilyIndices};
use crate::utils;

/// Intermediate struct to store the result of creating a Vulkan swapchain
struct VulkanSwapchain {
    loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

pub struct Swapchain {
    /// Handle to the logical device that this swapchain belongs to
    device: Rc<Device>,

    /// Manages the underlying Vulkan swapchain
    loader: ash::extensions::khr::Swapchain,
    /// Handle to Vulkan swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSwapchainKHR.html
    swapchain: vk::SwapchainKHR,

    /// Color format for all images
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html
    _format: vk::Format,
    /// Size, in pixels, of the swapchain
    pub extent: vk::Extent2D,

    /// Handle to Vulkan render pass being used by the graphics pipeline
    pub render_pass: vk::RenderPass,
    /// All framebuffers being used
    pub framebuffers: Vec<vk::Framebuffer>,

    /// Images that can be be drawn to and presented
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
    /// Index of frame being worked on (0 to number of framebuffers)
    current_frame: usize,
}

/// Constructors to create a swapchain
impl Swapchain {
    /// Creates a new swapchain
    ///
    /// Under the hood a new Vulkan swapchain is created as well the framebuffers, images, semaphores, and fences
    /// required to make the swapchain work
    pub fn new(device: Rc<Device>) -> Swapchain {
        let family_indices = Device::find_queue_family(
            &device.instance,
            device.physical_device,
            &device.surface_loader,
            &device.surface,
        );

        let vk_swapchain = Swapchain::create_swapchain(
            &device.instance,
            &device.device,
            device.physical_device,
            &device.surface_loader,
            &device.surface,
            &family_indices,
        );

        let image_views = Swapchain::create_image_views(
            &device.device,
            vk_swapchain.format,
            &vk_swapchain.images,
        );

        let render_pass = Swapchain::create_render_pass(&device.device, vk_swapchain.format);
        let framebuffers = Swapchain::create_framebuffers(
            &device.device,
            render_pass,
            &image_views,
            &vk_swapchain.extent,
        );

        let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Swapchain::create_sync_objects(&device.device);

        Swapchain {
            device,

            loader: vk_swapchain.loader,
            swapchain: vk_swapchain.swapchain,

            _format: vk_swapchain.format,
            extent: vk_swapchain.extent,

            _images: vk_swapchain.images,
            image_views,

            render_pass,
            framebuffers,

            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            current_frame: 0,
        }
    }

    /// Helper constructor to create a new Vulkan swapchain
    ///
    /// Returns a struct with the swapchain loader, created swapchain, images, format, and extent
    fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::extensions::khr::Surface,
        surface: &vk::SurfaceKHR,
        queue_family: &QueueFamilyIndices,
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
            .image_array_layers(1);

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

    /// Chooses the most optimal format for the application
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

    /// Chooses the most optimal present mode for the application
    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }

    /// Chooses the most optimal extent for the application
    fn choose_swapchain_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != std::u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: utils::constants::WINDOW_WIDTH.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: utils::constants::WINDOW_HEIGHT.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    /// Helper constructor to create an image view for every image in the swapchain
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

    /// Helper constructor that creates a new render pass
    fn create_render_pass(device: &ash::Device, surface_format: vk::Format) -> vk::RenderPass {
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

        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)
            .build()];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let render_pass_attachments = [color_attachment];
        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe {
            device
                .create_render_pass(&render_pass_info, None)
                .expect("Failed to create render pass")
        }
    }

    /// Helper constructor that creates a new framebuffer for every image in the swapchain
    fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let mut framebuffers: Vec<vk::Framebuffer> = Vec::new();
        for &image_view in image_views.iter() {
            let attachments = [image_view];

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

    /// Helper constructor that creates the required semaphores and fences for synchronization between the CPU and GPU
    fn create_sync_objects(
        device: &ash::Device,
    ) -> (Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>) {
        let mut sync_objects = (Vec::new(), Vec::new(), Vec::new());

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..utils::constants::MAX_FRAMES_IN_FLIGHT {
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

        sync_objects
    }
}

/// Public functions
impl Swapchain {
    /// Draws the newest framebuffer to the window
    pub fn draw_frame(&mut self, command_buffers: &Vec<vk::CommandBuffer>) {
        // Wait for previous frame to finish drawing (blocking wait)
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        let (image_index, _) = unsafe {
            self.device
                .device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for fences");

            self.loader
                .acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image")
        };

        // Get GPU to start working on the next frame
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let command_buffers = [command_buffers[image_index as usize]];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .device
                .reset_fences(&wait_fences)
                .expect("Failed to reset fences");

            self.device
                .device
                .queue_submit(
                    self.device.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit")
        };

        // Present the frame that just finished drawing
        let swapchains = [self.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.loader
                .queue_present(self.device.present_queue, &present_info)
                .expect("Failed to execute queue present");
        };

        self.current_frame = (self.current_frame + 1) % utils::constants::MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.image_views.iter() {
                self.device.device.destroy_image_view(image_view, None);
            }

            self.loader.destroy_swapchain(self.swapchain, None);

            for &framebuffer in self.framebuffers.iter() {
                self.device.device.destroy_framebuffer(framebuffer, None);
            }

            self.device
                .device
                .destroy_render_pass(self.render_pass, None);

            for i in 0..utils::constants::MAX_FRAMES_IN_FLIGHT {
                self.device
                    .device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device
                    .device
                    .destroy_fence(self.in_flight_fences[i], None);
            }
        };
    }
}
