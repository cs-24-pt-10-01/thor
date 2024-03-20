use std::ffi::{c_char, CStr};

// # Safety
//
// This function is unsafe because it dereferences the `id` pointer.
#[no_mangle]
pub unsafe extern "C" fn start_rapl(id: *const c_char) {}

// # Safety
//
// This function is unsafe because it dereferences the `id` pointer.
#[no_mangle]
pub unsafe extern "C" fn stop_rapl(id: *const c_char) {}
