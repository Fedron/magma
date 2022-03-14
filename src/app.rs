use ash::vk;
use std::ffi::{CStr, CString};
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
    _extent: vk::Extent2D,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);
        };
    }
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

/// Main application for Magma, and the entry point
pub struct App {
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
    _surface: Surface,

    /// Handle to Vulkan physical device this app is using
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDevice.html
    _physical_device: vk::PhysicalDevice,
    /// Handle to Vulkan logical device
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDevice.html
    device: ash::Device,

    /// Handle to Vulkan queue used for graphics operations
    _graphics_queue: vk::Queue,
    /// Handle to Vulkan queue used for presenting images
    _present_queue: vk::Queue,

    /// Handle to the current swapchain for rendering
    _swapchain: Swapchain,
    /// Handles to Vulkan image views for each image in the swapchain
    ///
    /// https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageView.html
    swapchain_image_views: Vec<vk::ImageView>,
}

impl App {
    /// Creates a new App
    ///
    /// Loads the Vulkan library and then creates a Vulkan instance
    pub fn new(window: &winit::window::Window) -> App {
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

        let _graphics_pipeline = App::create_graphics_pipeline(&device);

        App {
            _entry: entry,
            instance,
            debug_utils_loader,
            debug_messenger,
            _surface: surface,
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
            _swapchain: swapchain,
            swapchain_image_views,
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

        let queue_priorities = [1.0_f32];
        let queue_infos = [
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(indices.graphics_family.unwrap())
                .queue_priorities(&queue_priorities)
                .build(),
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(indices.present_family.unwrap())
                .queue_priorities(&queue_priorities)
                .build(),
        ];

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
            _extent: extent,
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

    fn create_graphics_pipeline(device: &ash::Device) {
        let shader_code = App::read_shader_code("shaders/simple-shader");
        let shader_module = App::create_shader_module(device, shader_code);

        let _shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_vs\0") })
                .stage(vk::ShaderStageFlags::VERTEX),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main_fs\0") })
                .stage(vk::ShaderStageFlags::FRAGMENT),
        ];

        unsafe {
            device.destroy_shader_module(shader_module, None);
        }
    }

    fn create_shader_module(device: &ash::Device, code: Vec<u32>) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code);
        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module")
        }
    }

    fn read_shader_code(shader_crate: &'static str) -> Vec<u32> {
        let shader_path = spirv_builder::SpirvBuilder::new(shader_crate, "spirv-unknown-vulkan1.0")
            .build()
            .unwrap()
            .module
            .unwrap_single()
            .to_path_buf();

        ash::util::read_spv(
            &mut std::fs::File::open(shader_path)
                .expect(&format!("Failed to open shader file {}", shader_crate)),
        )
        .expect(&format!(
            "Failed to read shader '{}' from spv",
            shader_crate
        ))
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

    pub fn draw_frame(&mut self) {}

    /// Runs the winit event loop, which wraps the App main loop
    pub fn main_loop(mut self, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            Event::MainEventsCleared => window.request_redraw(),
            Event::RedrawRequested(_) => self.draw_frame(),
            _ => {}
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);

            if utils::constants::ENABLE_VALIDATION_LAYERS {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
        };
    }
}
