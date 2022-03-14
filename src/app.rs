use ash::vk;
use std::{ffi::CString, path::Path};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::utils;

const WINDOW_TITLE: &'static str = "Magma";

/// Wrapper struct with all the queue families required for the app
struct QueueFamilyIndices {
    /// Index of the graphics queue family
    graphics_family: Option<u32>,
    /// Index of the present queue family
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    /// Returns whether or not all the queue family indices are present
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

/// Contains information about a Vulkan physical device, as well as a handle to the device
struct PhysicalDeviceInfo {
    name: String,
    _device_id: u32,
    device_type: String,
    is_suitable: bool,
    handle: vk::PhysicalDevice,
}

impl std::fmt::Display for PhysicalDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.device_type)
    }
}

/// Wrapper around a surface loader and a handle to a Vulkan surface
struct Surface {
    /// Manages the Vulkan surface
    pub loader: ash::extensions::khr::Surface,
    /// Handle to Vulkan surface used for this app's window
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceKHR.html
    pub surface: vk::SurfaceKHR,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        };
    }
}

/// Contains information about a Vulkan swapchain
struct Swapchain {
    /// Manages the underlying Vulkan swapchain
    loader: ash::extensions::khr::Swapchain,
    /// Handle to Vulkan swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSwapchainKHR.html
    swapchain: vk::SwapchainKHR,
    /// Images that can be be drawn to and presented
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImage.html
    images: Vec<vk::Image>,
    /// Color format for all images
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html
    format: vk::Format,
    /// Size, in pixels, of the swapchain
    extent: vk::Extent2D,
}

/// Contains information and the features and properties of a swapchain
struct SwapchainSupportInfo {
    /// Various properties of the swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceCapabilitiesKHR.html
    capabilities: vk::SurfaceCapabilitiesKHR,
    /// Supported color space and formats
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSurfaceFormatKHR.html
    formats: Vec<vk::SurfaceFormatKHR>,
    /// Supported present modes
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPresentModeKHR.html
    present_modes: Vec<vk::PresentModeKHR>,
}

/// Contains Vulkan synchronization objects for syncing the CPU and GPU
struct SynchronizationObjects {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
}

/// Main application for Magma, and the entry point
pub struct App {
    /// Handle to winit window
    window: winit::window::Window,

    /// Holds the loaded Vulkan library
    _entry: ash::Entry,
    /// Handle to the Vulkan instance
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkInstance.html
    instance: ash::Instance,
    /// Manages the debug_messenger
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    /// Handle to Vulkan debug messenger
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDebugUtilsMessengerEXT.html
    debug_messenger: vk::DebugUtilsMessengerEXT,
    /// Handle to the Vulkan surface and surface loader
    surface: Surface,

    /// Handle to Vulkan physical device this app is using
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html
    _physical_device: vk::PhysicalDevice,
    /// Handle to Vulkan logical device
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDevice.html
    device: ash::Device,

    /// Handle to Vulkan queue used for graphics operations
    graphics_queue: vk::Queue,
    /// Handle to Vulkan queue used for presenting images
    present_queue: vk::Queue,

    /// Handle to the current swapchain for rendering
    swapchain: Swapchain,
    /// Handles to Vulkan image views for each image in the swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageView.html
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    /// Handle to Vulkan render pass being used by the graphics pipeline
    render_pass: vk::RenderPass,
    /// Handle to Vulkan pipeline layout used by the graphics pipeline
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineLayout.html
    pipeline_layout: vk::PipelineLayout,
    /// Handle to Vulkan pipeline being used as a graphics pipeline
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipeline.html
    graphics_pipeline: vk::Pipeline,

    /// Handle to Vulkan command pool that contains all our command buffers
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCommandPool.html
    command_pool: vk::CommandPool,
    /// List of all the command buffers we have recorded
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCommandBuffer.html
    command_buffers: Vec<vk::CommandBuffer>,

    /// Wrapper for our Vulkan synchronization objection
    sync_objects: SynchronizationObjects,
    /// Index of frame being worked on (0 to number of framebuffers)
    current_frame: usize,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> App {
        let window = App::init_window(event_loop);

        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan library") };
        let instance = App::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) =
            utils::debug::setup_debug_utils(&entry, &instance);

        let surface = App::create_surface(&entry, &instance, &window);

        let physical_device = App::pick_physical_device(&instance, &surface);
        let (device, family_indices) =
            App::create_logical_device(&instance, physical_device, &surface);

        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let swapchain = App::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface,
            &family_indices,
        );
        let swapchain_image_views =
            App::create_image_views(&device, swapchain.format, &swapchain.images);

        let render_pass = App::create_render_pass(&device, swapchain.format);
        let (graphics_pipeline, pipeline_layout) =
            App::create_graphics_pipeline(&device, render_pass, swapchain.extent);

        let swapchain_framebuffers = App::create_framebuffers(
            &device,
            render_pass,
            &swapchain_image_views,
            &swapchain.extent,
        );

        let command_pool = App::create_command_pool(&device, &family_indices);
        let command_buffers = App::create_command_buffers(
            &device,
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swapchain.extent,
        );

        let sync_objects = App::create_sync_objects(&device);

        App {
            window,

            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            surface,

            _physical_device: physical_device,
            device,

            graphics_queue,
            present_queue,

            swapchain,
            swapchain_image_views,
            swapchain_framebuffers,

            render_pass,
            pipeline_layout,
            graphics_pipeline,

            command_pool,
            command_buffers,

            sync_objects,
            current_frame: 0,
        }
    }

    /// Constructor to create a Vulkan instance
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkInstance.html
    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        let required_extension_names = utils::platforms::required_extension_names();
        if !App::check_required_extensions(entry, &required_extension_names) {
            panic!("Missing extensions, see above");
        }

        if !utils::debug::check_validation_layer_support(
            entry,
            &utils::constants::VALIDATION_LAYERS,
        ) {
            panic!("Missing layers, see above");
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new("Magma").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .engine_name(&engine_name);

        let enabled_layer_names = if utils::constants::ENABLE_VALIDATION_LAYERS {
            Vec::new()
        } else {
            utils::constants::VALIDATION_LAYERS
                .iter()
                .map(|layer| layer.as_ptr() as *const i8)
                .collect::<Vec<*const i8>>()
        };
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&required_extension_names)
            .enabled_layer_names(&enabled_layer_names);

        unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create Vulkan instance")
        }
    }

    /// Checks if the Vulkan instance supports all the extensions we require
    ///
    /// Returns whether or not all required extensions are supported
    fn check_required_extensions(
        entry: &ash::Entry,
        required_extension_names: &Vec<*const i8>,
    ) -> bool {
        let supported_extension_names = entry
            .enumerate_instance_extension_properties(None)
            .expect("Failed to get instance extension properties");

        let is_missing_extensions = utils::contains_required(
            &supported_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &required_extension_names
                .iter()
                .map(|&extension| utils::char_ptr_to_string(extension))
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            panic!("Missing extensions, see above")
        }

        true
    }

    /// Creates a platform-specific surface for Vulkan
    ///
    /// Returns the surface loader and a handle to the create surface
    fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &Window) -> Surface {
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        let surface = unsafe {
            utils::platforms::create_surface(entry, instance, window)
                .expect("Failed to create surface")
        };

        Surface {
            loader: surface_loader,
            surface,
        }
    }

    /// Finds a Vulkan physical device that matches the needs of the application, and returns it
    fn pick_physical_device(instance: &ash::Instance, surface: &Surface) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate Vulkan physical devices")
        };

        let mut chosen_device: Option<PhysicalDeviceInfo> = None;
        for &physical_device in physical_devices.iter() {
            let physical_device_info =
                App::is_physical_device_suitable(instance, physical_device, surface);
            if physical_device_info.is_suitable {
                if chosen_device.is_none() {
                    chosen_device = Some(physical_device_info)
                }
            }
        }

        match chosen_device {
            Some(physical_device) => {
                log::info!("Using {}", physical_device);
                return physical_device.handle;
            }
            None => {
                log::error!("Failed to find a suitable GPU");
                panic!();
            }
        }
    }

    /// Checks a physical device for required features
    ///
    /// Returns whether or not the physical device is suitable
    fn is_physical_device_suitable(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> PhysicalDeviceInfo {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let _device_features = unsafe { instance.get_physical_device_features(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            _ => "Unknown",
        };

        let indices = App::find_queue_family(instance, physical_device, surface);

        let is_device_extensions_supported =
            App::check_device_extension_support(instance, physical_device);
        let is_swapchain_supported = if is_device_extensions_supported {
            let swapchain_support = App::query_swapchain_support(physical_device, surface);
            !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
        } else {
            false
        };

        PhysicalDeviceInfo {
            name: utils::char_array_to_string(&device_properties.device_name),
            _device_id: device_properties.device_id,
            device_type: String::from(device_type),
            is_suitable: indices.is_complete()
                && is_device_extensions_supported
                && is_swapchain_supported,
            handle: physical_device,
        }
    }

    /// Gets a physical device's queue families
    fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue_family_indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let has_present_support = unsafe {
                surface
                    .loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface.surface,
                    )
                    .expect("Failed to get surface support for physical device")
            };
            if queue_family.queue_count > 0 && has_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    /// Creates a Vulkan logical device from a physical device
    ///
    /// Returns a handle to the created logical device, and it's graphics queue
    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> (ash::Device, QueueFamilyIndices) {
        let indices = App::find_queue_family(instance, physical_device, surface);

        use std::collections::HashSet;
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_infos: Vec<vk::DeviceQueueCreateInfo> = Vec::new();
        for &queue_family in unique_queue_families.iter() {
            queue_infos.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family)
                    .queue_priorities(&queue_priorities)
                    .build(),
            );
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::default();

        let required_validation_layers: Vec<*const i8> = utils::constants::VALIDATION_LAYERS
            .iter()
            .map(|layer| layer.as_ptr() as *const i8)
            .collect();

        let device_extension_names_cstring: Vec<CString> = utils::constants::DEVICE_EXTENSIONS
            .iter()
            .map(|&extension| CString::new(extension).unwrap())
            .collect();

        let device_extension_names_ptr: Vec<*const i8> = device_extension_names_cstring
            .iter()
            .map(|t| t.as_ptr())
            .collect();

        let device_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_features(&physical_device_features)
            .enabled_layer_names(&required_validation_layers)
            .enabled_extension_names(&device_extension_names_ptr);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_info, None)
                .expect("Failed to create logical device")
        };

        (device, indices)
    }

    /// Checks if the physical device supports the required extensions
    ///
    /// Returns whether or not all required extensions are supported
    fn check_device_extension_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        let available_extension_names = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get instance device properties")
        };

        let is_missing_extensions = utils::contains_required(
            &available_extension_names
                .iter()
                .map(|extension| utils::char_array_to_string(&extension.extension_name))
                .collect::<Vec<String>>(),
            &utils::constants::DEVICE_EXTENSIONS
                .iter()
                .map(|&extension| extension.to_string())
                .collect::<Vec<String>>(),
        );

        if is_missing_extensions.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_extensions.1
            );
            panic!("Missing extensions, see above")
        }

        true
    }

    /// Gets the swapchain support info for a surface on a physical device
    fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
    ) -> SwapchainSupportInfo {
        unsafe {
            let capabilities = surface
                .loader
                .get_physical_device_surface_capabilities(physical_device, surface.surface)
                .expect("Failed to query for surface capabilities");

            let formats = surface
                .loader
                .get_physical_device_surface_formats(physical_device, surface.surface)
                .expect("Failed to query for surface formats");

            let present_modes = surface
                .loader
                .get_physical_device_surface_present_modes(physical_device, surface.surface)
                .expect("Failed to query for surface present modes");

            SwapchainSupportInfo {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    /// Creates a Vulkan swapchain
    fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: &Surface,
        queue_family: &QueueFamilyIndices,
    ) -> Swapchain {
        let swapchain_support = App::query_swapchain_support(physical_device, surface);
        let surface_format = App::choose_swapchain_format(&swapchain_support.formats);
        let present_mode = App::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = App::choose_swapchain_extent(&swapchain_support.capabilities);

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
            .surface(surface.surface)
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

        Swapchain {
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

    /// Creates an image view for every image
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

    /// Creates a new Vulkan graphics pipeline
    fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let vert_shader_code = App::read_shader_code(Path::new("shaders/spv/simple.vert.spv"));
        let vert_shader_module = App::create_shader_module(device, vert_shader_code);

        let frag_shader_code = App::read_shader_code(Path::new("shaders/spv/simple.frag.spv"));
        let frag_shader_module = App::create_shader_module(device, frag_shader_code);

        let main_function_name = CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(vert_shader_module)
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(frag_shader_module)
                .name(&main_function_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&[])
            .vertex_binding_descriptions(&[]);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_state_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL)
            .rasterizer_discard_enable(false)
            .depth_bias_clamp(0.0)
            .depth_bias_constant_factor(0.0)
            .depth_bias_enable(false)
            .depth_bias_slope_factor(0.0);

        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .sample_mask(&[])
            .alpha_to_one_enable(false)
            .alpha_to_coverage_enable(false);

        let stencil_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .compare_mask(0)
            .write_mask(0)
            .reference(0)
            .build();

        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state);

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ZERO)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build()];

        let color_blend_state_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&[])
            .push_constant_ranges(&[]);

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout")
        };

        let graphics_pipeline_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_state_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build()];

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &graphics_pipeline_infos,
                    None,
                )
                .expect("Failed to create graphics pipeline")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        };

        (graphics_pipeline[0], pipeline_layout)
    }

    /// Creates a new shader module from spirv code
    fn create_shader_module(device: &ash::Device, code: Vec<u8>) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
        };
        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }

    /// Reads a compiled spirv file from the path specified
    fn read_shader_code(shader_path: &Path) -> Vec<u8> {
        use std::fs::File;
        use std::io::Read;

        let spv_file = File::open(shader_path)
            .expect(&format!("Failed to find spv file at {:?}", shader_path));

        spv_file
            .bytes()
            .filter_map(|byte| byte.ok())
            .collect::<Vec<u8>>()
    }

    /// Creates a new render pass for a graphics pipeline
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

    /// Creates framebuffers for every swapchain image view
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

    /// Creates a new Vulkan command pool on a device
    fn create_command_pool(
        device: &ash::Device,
        queue_family: &QueueFamilyIndices,
    ) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family.graphics_family.unwrap());

        unsafe {
            device
                .create_command_pool(&create_info, None)
                .expect("Failed to create command pool")
        }
    }

    /// Creates and records new Vulkan command buffers for every framebuffer
    ///
    /// The command buffers only bind a graphics pipeline and draw 3 vertices
    fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.len() as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate command buffers")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording to command buffer")
            };

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(framebuffers[i])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                })
                .clear_values(&clear_values);

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline,
                );
                device.cmd_draw(command_buffer, 3, 1, 0, 0);

                device.cmd_end_render_pass(command_buffer);
                device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to finish recording command buffer");
            }
        }

        command_buffers
    }

    /// Creates Vulkan semaphores and fences to allow for synchronization between the CPU and GPU
    fn create_sync_objects(device: &ash::Device) -> SynchronizationObjects {
        let mut sync_objects = SynchronizationObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
        };

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..utils::constants::MAX_FRAMES_IN_FLIGHT {
            unsafe {
                sync_objects.image_available_semaphores.push(
                    device
                        .create_semaphore(&semaphore_info, None)
                        .expect("Failed to create semaphore"),
                );
                sync_objects.render_finished_semaphores.push(
                    device
                        .create_semaphore(&semaphore_info, None)
                        .expect("Failed to create semaphore"),
                );
                sync_objects.in_flight_fences.push(
                    device
                        .create_fence(&fence_info, None)
                        .expect("Failed to create semaphore"),
                );
            };
        }

        sync_objects
    }

    /// Initialises a winit window, returning the initialised window
    pub fn init_window(event_loop: &EventLoop<()>) -> Window {
        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(
                utils::constants::WINDOW_WIDTH,
                utils::constants::WINDOW_HEIGHT,
            ))
            .build(event_loop)
            .expect("")
    }

    /// Draws the newest framebuffer to the window
    pub fn draw_frame(&mut self) {
        // Wait for previous frame to finish drawing (blocking wait)
        let wait_fences = [self.sync_objects.in_flight_fences[self.current_frame]];

        let (image_index, _) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for fences");

            self.swapchain
                .loader
                .acquire_next_image(
                    self.swapchain.swapchain,
                    std::u64::MAX,
                    self.sync_objects.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image")
        };

        // Get GPU to start working on the next frame
        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];

        let command_buffers = [self.command_buffers[image_index as usize]];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build()];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset fences");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.sync_objects.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit")
        };

        // Present the frame that just finished drawing
        let swapchains = [self.swapchain.swapchain];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            self.swapchain
                .loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present");
        };

        self.current_frame = (self.current_frame + 1) % utils::constants::MAX_FRAMES_IN_FLIGHT;
    }

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => self.window.request_redraw(),
            Event::RedrawRequested(_) => self.draw_frame(),
            Event::LoopDestroyed => {
                unsafe {
                    self.device
                        .device_wait_idle()
                        .expect("Failed to wait until device idle");
                };
            }
            _ => {}
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for i in 0..utils::constants::MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.device
                    .destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);

            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain
                .loader
                .destroy_swapchain(self.swapchain.swapchain, None);
            self.device.destroy_device(None);
            self.surface
                .loader
                .destroy_surface(self.surface.surface, None);

            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        };
    }
}
