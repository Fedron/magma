use ash::{extensions::ext::DebugUtils, vk};

pub struct Debugger {
    debug_utils: DebugUtils,
    handle: vk::DebugUtilsMessengerEXT,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {}
    }
}

impl Debugger {
    pub fn check_validation_layers(entry: &ash::Entry) {
        let supported_layers = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to get instance layer properties");

        let is_missing_layers = crate::utils::contains_required(
            &supported_layers
                .iter()
                .map(|layer| crate::utils::char_array_to_string(&layer.layer_name))
                .collect::<Vec<String>>(),
            &required_validation_layers
                .iter()
                .map(|&layer| layer.to_string())
                .collect::<Vec<String>>(),
        );

        if is_missing_layers.0 {
            log::error!(
                "Your device is missing required extensions: {:?}",
                is_missing_layers.1
            );
            panic!("Missing extensions, see above")
        }
    }
}
