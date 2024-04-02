use std::ffi::CString;
use thor_lib_sync::ffi::{start_rapl, stop_rapl};

#[test]
fn test_send_function() {
    let func1 = CString::new("Function1").unwrap();
    unsafe { start_rapl(func1.as_ptr()) };
    unsafe { stop_rapl(func1.as_ptr()) };

    let func2 = CString::new("Function2").unwrap();
    unsafe { start_rapl(func2.as_ptr()) };
    unsafe { stop_rapl(func2.as_ptr()) };

    // Sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
