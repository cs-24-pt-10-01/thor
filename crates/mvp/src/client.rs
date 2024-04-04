use csv::{Writer, WriterBuilder};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    sync::{Mutex, Once},
    time::{SystemTime, UNIX_EPOCH},
};
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};

static RAPLMEASUREMENTS_HASHMAP: Mutex<Option<HashMap<(String, usize), (RaplMeasurement, u128)>>> =
    Mutex::new(None);
static CSV_WRITER: Mutex<Option<Writer<File>>> = Mutex::new(None);

static RAPL_INIT: Once = Once::new();

pub fn start_rapl(id: impl AsRef<str>) {
    RAPL_INIT.call_once(|| {
        // Initialize the RAPL hashmap
        *RAPLMEASUREMENTS_HASHMAP.lock().unwrap() = Some(HashMap::new());
    });

    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    // Insert the RAPL registers into the hashmap
    RAPLMEASUREMENTS_HASHMAP
        .lock()
        .expect("Failed to lock RAPL hashmap")
        .as_mut()
        .expect("RAPL hashmap is None")
        .insert(
            (id.as_ref().to_string(), thread_id::get()),
            (rapl_registers, timestamp),
        );

    /*
    match rapl_registers {
        RaplMeasurement::Intel(intel) => {
            write_to_csv(
                (
                    id.as_ref(),
                    timestamp,
                    intel.pp0,
                    intel.pp1,
                    intel.pkg,
                    intel.dram,
                ),
                ["id", "timestamp", "pp0", "pp1", "pkg", "dram"],
            )
            .unwrap();
        }
        RaplMeasurement::AMD(amd) => {
            write_to_csv(
                (id.as_ref(), timestamp, amd.core, amd.pkg),
                ["id", "timestamp", "core", "pkg"],
            )
            .unwrap();
        }
    }
    */
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let stop_rapl_registers = read_rapl_msr_registers();

    let stop_timestamp = get_timestamp_millis();

    // Get the RAPL registers from the hashmap
    let (start_rapl_registers, start_timestamp) = RAPLMEASUREMENTS_HASHMAP
        .lock()
        .expect("Failed to lock RAPL hashmap")
        .as_mut()
        .expect("RAPL hashmap is None")
        .remove(&(id.as_ref().to_string(), thread_id::get()))
        .expect("Failed to remove RAPL registers from hashmap");

    match (start_rapl_registers, stop_rapl_registers) {
        (RaplMeasurement::Intel(start_intel), RaplMeasurement::Intel(stop_intel)) => {
            write_to_csv(
                (
                    id.as_ref(),
                    start_timestamp,
                    stop_timestamp,
                    stop_intel.pp0 - start_intel.pp0,
                    stop_intel.pp1 - start_intel.pp1,
                    stop_intel.pkg - start_intel.pkg,
                    stop_intel.dram - start_intel.dram,
                ),
                [
                    "id",
                    "start_timestamp",
                    "stop_timestamp",
                    "pp0",
                    "pp1",
                    "pkg",
                    "dram",
                ],
            )
            .unwrap();
        }
        (RaplMeasurement::AMD(start_amd), RaplMeasurement::AMD(stop_amd)) => {
            write_to_csv(
                (
                    id.as_ref(),
                    start_timestamp,
                    stop_timestamp,
                    stop_amd.core - start_amd.core,
                    stop_amd.pkg - start_amd.pkg,
                ),
                ["id", "start_timestamp", "stop_timestamp", "core", "pkg"],
            )
            .unwrap();
        }
        _ => panic!("RaplMeasurement types do not match"),
    }

    /*
    match rapl_registers {
        RaplMeasurement::Intel(intel) => {
            write_to_csv(
                (
                    id.as_ref(),
                    timestamp,
                    intel.pp0,
                    intel.pp1,
                    intel.pkg,
                    intel.dram,
                ),
                ["id", "timestamp", "pp0", "pp1", "pkg", "dram"],
            )
            .unwrap();
        }
        RaplMeasurement::AMD(amd) => {
            write_to_csv(
                (id.as_ref(), timestamp, amd.core, amd.pkg),
                ["id", "timestamp", "core", "pkg"],
            )
            .unwrap();
        }
    }
    */
}

fn write_to_csv<T, C, U>(data: T, columns: C) -> Result<(), std::io::Error>
where
    T: Serialize,
    C: IntoIterator<Item = U>,
    U: AsRef<[u8]>,
{
    // Lock the CSV writer
    let mut wtr_lock = CSV_WRITER.lock().unwrap();

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
    wtr.flush()?;

    Ok(())
}

fn get_timestamp_millis() -> u128 {
    let current_time = SystemTime::now();
    let duration_since_epoch = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    duration_since_epoch.as_millis()
}
