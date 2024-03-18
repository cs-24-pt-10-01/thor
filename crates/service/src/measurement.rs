use anyhow::Result;
use crossbeam::queue::SegQueue;
use rangemap::RangeMap;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    ptr::NonNull,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};
use thor_shared::{ConnectionType, LocalClientPacket, RemoteClientPacket};
use tokio::{io::AsyncReadExt, net::TcpListener};

use crate::component_def::{Build, Listener, Measurement, StartProcess};

// TODO: The usage of VecDeque in the codebase seems to reflect that of a ringbuffer. Rather than using a static mut along with using unsafe, it could be checked if the implementations are equivalent.
static mut SAMPLING_THREAD_DATA: VecDeque<(RaplMeasurement, u128)> = VecDeque::new();
pub struct RaplSampler {
    pub max_sample_age: u128,
}

impl Measurement<RaplMeasurement> for RaplSampler {
    fn get_measurement(&self, timestamp: u128) -> RaplMeasurement {
        let result = find_measurement(timestamp);

        //deleting old measurements
        unsafe {
            while let Some((_, time)) = SAMPLING_THREAD_DATA.front() {
                if *time < timestamp - self.max_sample_age {
                    SAMPLING_THREAD_DATA.pop_front();
                } else {
                    break;
                }
            }
        }

        result
    }
}

impl RaplSampler {
    pub fn start_sampling(&self, sampling_interval: u64) -> Result<()> {
        thread::spawn(move || {
            rapl_sampling_thread(sampling_interval);
        });
        Ok(())
    }
}

fn find_measurement(timestamp: u128) -> RaplMeasurement {
    unsafe {
        let mut last: &RaplMeasurement = &SAMPLING_THREAD_DATA.back().unwrap().0;
        for (rapl_measurement, time) in SAMPLING_THREAD_DATA.iter().rev() {
            if *time <= timestamp {
                return last.clone();
            }
            last = rapl_measurement;
        }
        panic!("No measurement found for timestamp {}", timestamp);
    }
}

fn rapl_sampling_thread(sampling_interval: u64) {
    // Loop and sample the RAPL data
    loop {
        // Grab the RAPL data and the timestamp, then push it to the queue
        let rapl_measurement = read_rapl_msr_registers();
        let timestamp = get_timestamp_millis();
        unsafe {
            SAMPLING_THREAD_DATA.push_back((rapl_measurement, timestamp));
        }

        // Sleep for the sampling interval
        thread::sleep(Duration::from_micros(sampling_interval));
    }
}

fn get_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

// TODO: Consider handling for process usage

// Create a system and refresh it
// TODO: Maybe move this into the main function initially,
// and then pass it to this function, to prevent receiving packets before it is ready
/*let mut sys = System::new_all();
sys.refresh_all();

thread::sleep(Duration::from_secs(5));
//std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

for i in 0..5 {
    sys.refresh_processes_specifics(ProcessRefreshKind::everything().with_cpu());

    // Print all proceeses and their CPU usage
    for (pid, process) in sys.processes() {
        if process.cpu_usage() > 0.0 {
            println!(
                "Iteration: {}, name: {}, exe: {:?}, pid: {}, cpu usage: {:?}, memory: {}, status: {:?}",
                i,
                process.name(),
                process.exe(),
                process.pid(),
                process.cpu_usage(),
                process.memory(),
                process.status(),
            );
        }
    }

    // Print status of the WoW Classic process
    for process in sys.processes_by_exact_name("WowClassic.exe") {
        println!(
            "WoW Classic process: name: {}, exe: {:?}, pid: {}, cpu usage: {:?}, memory: {}, status: {:?}",
            process.name(),
            process.exe(),
            process.pid(),
            process.cpu_usage(),
            process.memory(),
            process.status(),
        );
    }

    // Sleep for the minimum CPU update interval
    thread::sleep(Duration::from_secs(10));
}*/
