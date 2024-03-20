use std::ffi::CString;
use thor_minimal_client_sync::ffi::{start_rapl, stop_rapl};

#[test]
fn test_thor_minimal_client_sync_1000_st() {
    let func1 = CString::new("Function1").unwrap();

    for _ in 0..1000 {
        unsafe { start_rapl(func1.as_ptr()) };
        unsafe { stop_rapl(func1.as_ptr()) };
    }

    // Sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[test]
fn test_thor_minimal_client_sync_1000_mt() {
    let func1 = CString::new("Function1").unwrap();
    let func2 = CString::new("Function2").unwrap();
    let func3 = CString::new("Function3").unwrap();
    let func4 = CString::new("Function4").unwrap();

    let handles = (0..250)
        .map(|_| {
            let func1 = func1.clone();
            let func2 = func2.clone();
            let func3 = func3.clone();
            let func4 = func4.clone();

            std::thread::spawn(move || {
                unsafe { start_rapl(func1.as_ptr()) };
                unsafe { stop_rapl(func1.as_ptr()) };

                unsafe { start_rapl(func2.as_ptr()) };
                unsafe { stop_rapl(func2.as_ptr()) };

                unsafe { start_rapl(func3.as_ptr()) };
                unsafe { stop_rapl(func3.as_ptr()) };

                unsafe { start_rapl(func4.as_ptr()) };
                unsafe { stop_rapl(func4.as_ptr()) };
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.join().unwrap();
    }

    // Sleep for 5 seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
}
