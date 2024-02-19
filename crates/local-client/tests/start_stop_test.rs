use std::ffi::CString;
use thor_local_client::ffi::{start_rapl, stop_rapl};

#[test]
fn test_send_function() {
    let func = CString::new("MyCoolFunc").unwrap();
    unsafe { start_rapl(func.as_ptr()) };

    let func = CString::new("MyCoolFunction").unwrap();
    unsafe { stop_rapl(func.as_ptr()) };

    // Sleep for 10 seconds
    std::thread::sleep(std::time::Duration::from_secs(10));
}
