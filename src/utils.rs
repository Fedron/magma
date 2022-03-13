use std::{ffi::CStr, os::raw::c_char};

pub mod constants;
pub mod debug;
pub mod platforms;

/// Converts a char array to a String
pub fn char_array_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe { CStr::from_ptr(raw_string_array.as_ptr()) };

    raw_string
        .to_str()
        .expect("Failed to convert char array to String")
        .to_owned()
}

pub fn char_ptr_to_string(string_ptr: *const i8) -> String {
    let raw_string = unsafe { CStr::from_ptr(string_ptr) };

    raw_string
        .to_str()
        .expect("Failed to convert char array to String")
        .to_owned()
}
