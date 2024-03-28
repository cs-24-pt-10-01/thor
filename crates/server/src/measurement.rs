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

static SAMPLING_THREAD_DATA: SegQueue<(RaplMeasurement, u128)> = SegQueue::new();

pub struct RaplSampler {
    pub max_sample_age: u128,
    range_map: RangeMap<u128, RaplMeasurement>,
    sampling_interval: u64,
}

impl Measurement<RaplMeasurement> for RaplSampler {
    fn get_measurement(&mut self, timestamp: u128) -> RaplMeasurement {
        self.update_range_map(timestamp);

        let result = self
            .range_map
            .get(&timestamp)
            .expect("No measurement found");

        result.clone()
    }

    fn get_multiple_measurements(&mut self, timestamps: &Vec<u128>) -> Vec<&RaplMeasurement> {
        let mut result = Vec::new();

        // updating rangemap using the first timestamp
        self.update_range_map(timestamps[0]);

        // find measurements
        for timestamp in timestamps {
            let measurement = self.range_map.get(timestamp).expect("No measurement found");
            result.push(measurement);
        }

        result
    }
}

impl RaplSampler {
    pub fn new(max_sample_age: u128, sampling_interval: u64) -> RaplSampler {
        let result = RaplSampler {
            max_sample_age,
            range_map: RangeMap::new(),
            sampling_interval,
        };
        result.start_sampling(sampling_interval).unwrap();
        result
    }

    fn start_sampling(&self, sampling_interval: u64) -> Result<()> {
        thread::spawn(move || {
            rapl_sampling_thread(sampling_interval);
        });
        Ok(())
    }

    fn update_range_map(&mut self, timestamp: u128) {
        // add new measurements
        while let Some((measurement, time)) = SAMPLING_THREAD_DATA.pop() {
            self.range_map
                .insert(time..time + self.sampling_interval as u128, measurement);
        }

        //remove old measurements
        self.range_map.remove(0..timestamp - self.max_sample_age);
    }
}

fn rapl_sampling_thread(sampling_interval: u64) {
    // Loop and sample the RAPL data
    loop {
        // Grab the RAPL data and the timestamp, then push it to the queue
        let rapl_measurement = read_rapl_msr_registers();
        let timestamp = get_timestamp_millis();

        SAMPLING_THREAD_DATA.push((rapl_measurement, timestamp));

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