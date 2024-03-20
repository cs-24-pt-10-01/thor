use std::ffi::CString;
use thor_minimal_client_sync::ffi::{start_rapl, stop_rapl};

#[test]
fn test_thor_minimal_client_sync_1000() {
    let func1 = CString::new("Function1").unwrap();
    let func2 = CString::new("Function2").unwrap();

    for _ in 0..1000 {
        unsafe { start_rapl(func1.as_ptr()) };
        unsafe { stop_rapl(func1.as_ptr()) };

        unsafe { start_rapl(func2.as_ptr()) };
        unsafe { stop_rapl(func2.as_ptr()) };
    }

    // Sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
