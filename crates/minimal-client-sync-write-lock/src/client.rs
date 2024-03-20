use csv::{Writer, WriterBuilder};
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    time::{SystemTime, UNIX_EPOCH},
};
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};

// Need extra examples:
// minimal-client-async-write-lock
// minimal-client-async-write-lockfree

// TODO: Need to lock here because there can be multiple threads trying to access the same writer
static mut CSV_WRITER: Option<Writer<File>> = None;

pub fn start_rapl(id: impl AsRef<str>) {
    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    match rapl_registers {
        RaplMeasurement::Intel(intel) => {
            write_to_csv(intel, vec!["pp0", "pp1", "pkg", "dram"]).unwrap();
        }
        RaplMeasurement::AMD(amd) => {
            write_to_csv(amd, vec!["core", "pkg"]).unwrap();
        }
    }
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let rapl_registers = read_rapl_msr_registers();

    let timestamp = get_timestamp_millis();

    match rapl_registers {
        RaplMeasurement::Intel(intel) => {
            write_to_csv(intel, vec!["pp0", "pp1", "pkg", "dram"]).unwrap();
        }
        RaplMeasurement::AMD(amd) => {
            write_to_csv(amd, vec!["core", "pkg"]).unwrap();
        }
    }
}

fn write_to_csv<T, C, U>(data: T, columns: C) -> Result<(), std::io::Error>
where
    T: Serialize,
    C: IntoIterator<Item = U>,
    U: AsRef<[u8]>,
{
    let wtr = match unsafe { CSV_WRITER.as_mut() } {
        Some(wtr) => wtr,
        None => {
            // Open the file to write to CSV. First argument is CPU type, second is RAPL power units
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(format!("test.csv",))?;

            // Create the CSV writer
            let mut wtr = WriterBuilder::new().from_writer(file);

            // Write the column names
            wtr.write_record(columns)?;

            // Store the CSV writer in a static variable
            unsafe { CSV_WRITER = Some(wtr) };

            // Return a mutable reference to the CSV writer
            unsafe { CSV_WRITER.as_mut().expect("failed to get CSV writer") }
        }
    };

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
