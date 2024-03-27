use csv::{Writer, WriterBuilder};
use serde::Serialize;
use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    sync::{Mutex, Once},
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

static QUEUE: Mutex<VecDeque<(RaplMeasurement, u128)>> = Mutex::new(VecDeque::new());

static RAPL_INIT: Once = Once::new();

pub fn start_rapl(id: impl AsRef<str>) {
    RAPL_INIT.call_once(|| {
        std::thread::spawn(|| background_writer);
    });

    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    QUEUE.lock().unwrap().push_back((rapl_registers, timestamp));
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    QUEUE.lock().unwrap().push_back((rapl_registers, timestamp));
}

fn background_writer() {
    println!("Starting background writer");

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("rapl_data.csv")
        .unwrap();

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

    while QUEUE.lock().unwrap().is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    wtr.write_record(["id", "a", "b", "c"]).unwrap();

    loop {
        let mut queue = QUEUE.lock().unwrap();

        while let Some((rapl_registers, timestamp)) = queue.pop_front() {
            match rapl_registers {
                RaplMeasurement::Intel(intel) => {
                    wtr.serialize(("id", timestamp, intel.pp0, intel.pp1, intel.pkg, intel.dram))
                        .unwrap();
                }
                RaplMeasurement::AMD(amd) => {
                    wtr.serialize(("id", timestamp, amd.core, amd.pkg)).unwrap();
                }
            }
        }
        // sleep 250ms
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn write_to_csv<T, C, U>(data: T, columns: C) -> Result<(), std::io::Error>
where
    T: Serialize,
    C: IntoIterator<Item = U>,
    U: AsRef<[u8]>,
{
    // Lock the CSV writer
    /*let mut wtr_lock = CSV_WRITER.lock().unwrap();

    // Check if mutex is none
    if wtr_lock.is_none() {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("rapl_data.csv")?;

        let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

        // Write the headers to the CSV
        wtr.write_record(columns)?;

        // Set the CSV writer
        *wtr_lock = Some(wtr);
    }

    let wtr = wtr_lock.as_mut().unwrap();

    // Write the data to the CSV and flush it
    wtr.serialize(data)?;
    wtr.flush()?;*/

    Ok(())
}

fn get_timestamp_millis() -> u128 {
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    duration_since_epoch.as_millis()
}
