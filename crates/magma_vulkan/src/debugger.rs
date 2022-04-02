use ash::{extensions::ext::DebugUtils, vk};

pub struct Debugger {
    handle: vk::DebugUtilsMessengerEXT,
    debug_utils: DebugUtils,
}
