use crossbeam::queue::SegQueue;
use csv::{Writer, WriterBuilder};
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    net::TcpStream,
    sync::Once,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};
use thor_shared::{ConnectionType, LocalClientPacket, LocalClientPacketOperation};

static LOCAL_CLIENT_PACKET_QUEUE: SegQueue<LocalClientPacket> = SegQueue::new();
static mut CSV_WRITER: Option<Writer<File>> = None;

pub fn start_rapl(id: impl AsRef<str>) {
    let rapl_registers = read_rapl_msr_registers();

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
    let packet = LocalClientPacket {
        id: id.as_ref().to_string(),
        process_id: 12345,
        thread_id: thread_id::get(),
        operation: LocalClientPacketOperation::Stop,
        timestamp: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };
    LOCAL_CLIENT_PACKET_QUEUE.push(packet);
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
