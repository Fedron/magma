use std::{collections::HashSet, ffi::CStr, os::raw::c_char};

/// Converts a char array to a String
pub fn char_array_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe { CStr::from_ptr(raw_string_array.as_ptr()) };

    raw_string
        .to_str()
        .expect("Failed to convert char array to String")
        .to_owned()
}

/// Converts a char pointer to a String
pub fn char_ptr_to_string(string_ptr: *const i8) -> String {
    let raw_string = unsafe { CStr::from_ptr(string_ptr) };

    raw_string
        .to_str()
        .expect("Failed to convert char array to String")
        .to_owned()
}

/// Checks whether a vector contains all of the required vector
///
/// Returns whether `to_check` contains all of required, and a vector of the missing items
pub fn contains_required(to_check: &Vec<String>, required: &Vec<String>) -> (bool, Vec<String>) {
    let required_hash_set = HashSet::<String>::from_iter(required.clone());
    let to_check_hash_set = &HashSet::<String>::from_iter(to_check.clone());
    let missing_required = required_hash_set
        .difference(to_check_hash_set)
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();

    if missing_required.len() > 0 {
        log::error!(
            "Your device is missing required features: {:?}",
            missing_required
        );
        return (true, missing_required);
    }

    (false, Vec::new())
}