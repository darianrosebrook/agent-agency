/// Stub implementations for vision functions (for testing when Swift bridge is not available)

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn vision_extract_text(
    _image_path: *const std::ffi::c_char,
    out_text: *mut *mut std::ffi::c_char,
    out_confidence: *mut f32,
    out_error: *mut *mut std::ffi::c_char,
) -> std::ffi::c_int {
    // Stub implementation - return placeholder text
    let text = std::ffi::CString::new("Placeholder vision text extraction").unwrap();
    unsafe {
        *out_text = text.into_raw();
        *out_confidence = 0.8;
        *out_error = std::ptr::null_mut();
    }
    0 // Success
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn vision_free_string(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}




