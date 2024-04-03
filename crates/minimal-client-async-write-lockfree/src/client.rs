use crossbeam::queue::SegQueue;
use csv::WriterBuilder;
use std::{
    collections::VecDeque,
    fs::OpenOptions,
    sync::Once,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};

// Context:
// This is an example of rapl-interface that is intended to be used in an application that can have mulitple threads calling to start_rapl and stop_rapl.
// The design of rapl-interface is insufficient due to its lack of thread safety. The CSV_WRITER is a static variable that is shared between threads and is not protected by a lock.
// This design is changed to perform thread safe operations by using a lock to protect the CSV_WRITER.

// Need extra examples: (pass to queue (MPMC design), then write to file)
// minimal-client-async-write-lock (try with a VecDeque that uses a lock)
// minimal-client-async-write-lockfree (use a lockfree data structure such as a queue)

static QUEUE: SegQueue<(String, RaplMeasurement, u128)> = SegQueue::new();

static RAPL_INIT: Once = Once::new();

pub fn start_rapl(id: impl AsRef<str>) {
    RAPL_INIT.call_once(|| {
        thread::spawn(background_writer);
    });

    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    QUEUE.push((id.as_ref().to_string(), rapl_registers, timestamp));
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    QUEUE.push((id.as_ref().to_string(), rapl_registers, timestamp));
}

static WRITER_INIT: Once = Once::new();

fn background_writer() {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("rapl_data.csv")
        .unwrap();

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

    while QUEUE.is_empty() {
        thread::sleep(std::time::Duration::from_millis(250));
    }

    let mut data = VecDeque::new();
    loop {
        while let Some(measurement) = QUEUE.pop() {
            data.push_back(measurement);
        }

        while let Some((id, rapl_registers, timestamp)) = data.pop_front() {
            WRITER_INIT.call_once(|| match rapl_registers {
                RaplMeasurement::Intel(_) => {
                    wtr.write_record(["id", "timestamp", "pp0", "pp1", "pkg", "dram"])
                        .unwrap();
                }
                RaplMeasurement::AMD(_) => {
                    wtr.write_record(["id", "timestamp", "core", "pkg"])
                        .unwrap();
                }
            });

            match rapl_registers {
                RaplMeasurement::Intel(intel) => {
                    wtr.serialize((id, timestamp, intel.pp0, intel.pp1, intel.pkg, intel.dram))
                        .unwrap();
                }
                RaplMeasurement::AMD(amd) => {
                    wtr.serialize((id, timestamp, amd.core, amd.pkg)).unwrap();
                }
            }
        }

        wtr.flush().unwrap();

        data.clear();

        // sleep 250ms
        thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn get_timestamp_millis() -> u128 {
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    duration_since_epoch.as_millis()
}
