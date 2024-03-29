use std::ffi::CString;
use thor_minimal_client_async_lockfree::ffi::{start_rapl, stop_rapl};

#[test]
fn test_thor_minimal_client_async_lockfree_1000_st() {
    let func1 = CString::new("FunctionST").unwrap();

    for _ in 0..1000 {
        unsafe { start_rapl(func1.as_ptr()) };
        unsafe { stop_rapl(func1.as_ptr()) };
    }

    // sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}

// cargo test --release --package thor-minimal-client-async-lockfree --test test_minimal_client_async_write_lockfree -- test_thor_minimal_client_async_lockfree_1000_mt --exact --nocapture
#[test]
fn test_thor_minimal_client_async_lockfree_1000_mt() {
    let func1 = CString::new("FunctionMT").unwrap();

    // Test for 8 threads 1000 times
    let handles = (0..8)
        .map(|_| {
            let func1 = func1.clone();
            std::thread::spawn(move || {
                for _ in 0..10000 {
                    unsafe { start_rapl(func1.as_ptr()) };
                    unsafe { stop_rapl(func1.as_ptr()) };
                }
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    // sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
