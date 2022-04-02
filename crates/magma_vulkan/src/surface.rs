use ash::vk;

pub struct Surface {
    handle: vk::SurfaceKHR,
    loader: ash::extensions::khr::Surface,
}
