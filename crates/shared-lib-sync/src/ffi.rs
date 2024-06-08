use crate::library;
use std::ffi::{c_char, CStr};

/// # Safety
///
/// This function is unsafe because it dereferences the `id` pointer.
#[no_mangle]
pub unsafe extern "C" fn start_rapl(id: *const c_char) {
    let id_cstr = CStr::from_ptr(id);
    let id_string = String::from_utf8_lossy(id_cstr.to_bytes()).to_string();

    library::start_rapl(id_string);
}

/// # Safety
///
/// This function is unsafe because it dereferences the `id` pointer.
#[no_mangle]
pub unsafe extern "C" fn stop_rapl(id: *const c_char) {
    let id_cstr = CStr::from_ptr(id);
    let id_string = String::from_utf8_lossy(id_cstr.to_bytes()).to_string();

    library::stop_rapl(id_string);
}
