use std::ffi::CString;
use thor_minimal_client_async_lock::ffi::{start_rapl, stop_rapl};

#[test]
fn test_thor_minimal_client_sync_1000_st() {
    let func1 = CString::new("Function1").unwrap();

    for _ in 0..1000 {
        unsafe { start_rapl(func1.as_ptr()) };
        unsafe { stop_rapl(func1.as_ptr()) };
    }
}

#[test]
fn test_thor_minimal_client_sync_1000_mt() {
    let func1 = CString::new("Function1").unwrap();

    // Test for 8 threads 1000 times
    let handles = (0..8)
        .map(|_| {
            let func1 = func1.clone();
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    unsafe { start_rapl(func1.as_ptr()) };
                    unsafe { stop_rapl(func1.as_ptr()) };
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }
}
