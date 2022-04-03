use std::rc::Rc;

use ash::vk;

use crate::device::LogicalDevice;

struct VulkanSwapchain {
    handle: vk::SwapchainKHR,
    swapchain: ash::extensions::khr::Swapchain,

    images: Vec<vk::Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

pub struct Swapchain {
    handle: vk::SwapchainKHR,
    device: Rc<LogicalDevice>,
    swapchain: ash::extensions::khr::Swapchain,

    render_pass: vk::RenderPass,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    depth_images: Vec<vk::Image>,
    depth_image_memories: Vec<vk::DeviceMemory>,
    depth_image_views: Vec<vk::ImageView>,

    color_format: vk::Format,
    depth_format: vk::Format,
    extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(logical_device: Rc<LogicalDevice>) -> Swapchain {
        let vk_swapchain = Swapchain::create_swapchain(logical_device.as_ref(), None);
        let render_pass =
            Swapchain::create_render_pass(logical_device.as_ref(), vk_swapchain.format);
        let image_views = Swapchain::create_image_views(
            logical_device.vk_handle(),
            vk_swapchain.format,
            &vk_swapchain.images,
        );
        let (depth_images, depth_image_memories, depth_image_views) =
            Swapchain::create_depth_resources(
                logical_device.as_ref(),
                image_views.len(),
                vk_swapchain.extent,
            );

        let depth_format = Swapchain::find_depth_format(logical_device.as_ref());

        Swapchain {
            handle: vk_swapchain.handle,
            device: logical_device,
            swapchain: vk_swapchain.swapchain,

            render_pass,
            images: vk_swapchain.images,
            image_views,
            depth_images,
            depth_image_memories,
            depth_image_views,

            color_format: vk_swapchain.format,
            depth_format,
            extent: vk_swapchain.extent,
        }
    }

    fn create_swapchain(
        logical_device: &LogicalDevice,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> VulkanSwapchain {
        let swapchain_support = logical_device.physical_device().swapchain_support();
        let surface_format = Swapchain::choose_format(&swapchain_support.formats);
        let present_mode = Swapchain::choose_present_mode(&swapchain_support.present_modes);
        let extent = Swapchain::choose_extent(&swapchain_support.capabilities);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            swapchain_support.capabilities.max_image_count
        } else {
            image_count
        };

        let queue_family_indices = logical_device.physical_device().indices();
        let (image_sharing_mode, queue_family_indices) =
            if queue_family_indices.graphics_family != queue_family_indices.present_family {
                (
                    vk::SharingMode::CONCURRENT,
                    vec![
                        queue_family_indices.graphics_family.unwrap(),
                        queue_family_indices.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, Vec::new())
            };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(logical_device.surface().vk_handle())
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

        let swapchain = ash::extensions::khr::Swapchain::new(
            logical_device.instance().vk_handle(),
            logical_device.vk_handle(),
        );
        let handle = unsafe {
            swapchain
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain")
        };

        let images = unsafe {
            swapchain
                .get_swapchain_images(handle)
                .expect("Failed to get swapchain images")
        };

        VulkanSwapchain {
            handle,
            swapchain,

            images,
            format: surface_format.format,
            extent,
        }
    }

    fn choose_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        for &available_format in available_formats.iter() {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format;
            }
        }

        *available_formats.first().unwrap()
    }

    fn choose_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
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

    fn create_render_pass(device: &LogicalDevice, surface_format: vk::Format) -> vk::RenderPass {
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
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_pass_attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        unsafe {
            device
                .vk_handle()
                .create_render_pass(&create_info, None)
                .expect("Failed to create render pass")
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

    fn create_depth_resources(
        device: &LogicalDevice,
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
                    .vk_handle()
                    .create_image_view(&image_view_info, None)
                    .expect("Failed to create depth image view")
            });
            depth_images.push(depth_image);
            depth_image_memories.push(depth_image_memory);
        }

        (depth_images, depth_image_memories, depth_image_views)
    }

    fn find_depth_format(device: &LogicalDevice) -> vk::Format {
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
}

impl Swapchain {
    pub fn vk_handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    pub fn images(&self) -> &[vk::Image] {
        &self.images
    }

    pub fn image_views(&self) -> &[vk::ImageView] {
        &self.image_views
    }

    pub fn color_format(&self) -> &vk::Format {
        &self.color_format
    }

    pub fn depth_format(&self) -> &vk::Format {
        &self.depth_format
    }

    pub fn extent(&self) -> &vk::Extent2D {
        &self.extent
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.image_views.iter() {
                self.device.vk_handle().destroy_image_view(image_view, None);
            }

            self.swapchain.destroy_swapchain(self.handle, None);

            for i in 0..self.depth_images.len() {
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

            self.device
                .vk_handle()
                .destroy_render_pass(self.render_pass, None);
        };
    }
}
