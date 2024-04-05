use std::ffi::CString;
use thor_mvp::ffi::{start_rapl, stop_rapl};

// cargo test --release --package thor-mvp --test test_mvp -- test_mvp_1000_st --exact --nocapture
#[test]
fn test_mvp_1000_st() {
    let func1 = CString::new("FunctionST").unwrap();

    for _ in 0..1000 {
        unsafe { start_rapl(func1.as_ptr()) };
        unsafe { stop_rapl(func1.as_ptr()) };
    }
}

// cargo test --release --package thor-mvp --test test_mvp -- test_mvp_1000_mt --exact --nocapture
#[test]
fn test_mvp_1000_mt() {
    let func1 = CString::new("FunctionMT").unwrap();

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

// cargo test --release --package thor-mvp --test test_mvp -- test_mvp_infinite_st --exact --nocapture
#[test]
fn test_mvp_infinite_st() {
    let func1 = CString::new("FunctionMT").unwrap();

    loop {
        // TODO: Can also just disable conversion to joules and get raw values, then check the file manually. Do it if no overflow occurs
        unsafe { start_rapl(func1.as_ptr()) };
        std::thread::sleep(std::time::Duration::from_secs(60));
        unsafe { stop_rapl(func1.as_ptr()) };
    }
}
