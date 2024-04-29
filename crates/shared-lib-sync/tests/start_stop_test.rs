use shared_lib_sync::ffi::{start_rapl, stop_rapl};
use std::ffi::CString;

#[test]
fn test_send_function() {
    let func1 = CString::new("Function1").unwrap();
    unsafe { start_rapl(func1.as_ptr()) };
    unsafe { stop_rapl(func1.as_ptr()) };

    let func2 = CString::new("Function2").unwrap();
    unsafe { start_rapl(func2.as_ptr()) };
    unsafe { stop_rapl(func2.as_ptr()) };
}
