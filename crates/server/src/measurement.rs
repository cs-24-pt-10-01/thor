use crate::component_def::Measurement;
use anyhow::Result;
use crossbeam::queue::SegQueue;
use rangemap::RangeMap;
use std::{
    thread,
    time::{Duration, SystemTime},
};
use thor_lib::{
    convert_to_joules, read_rapl_msr_registers, RaplMeasurement, RaplMeasurementJoules,
};

static SAMPLING_THREAD_DATA: SegQueue<(RaplMeasurement, u128)> = SegQueue::new();

pub struct RaplSampler {
    pub max_sample_age: u128,
    range_map: RangeMap<u128, (RaplMeasurement, u32)>,
    sampling_interval: u64,
    pkg_overflow: u32,
    last_pkg: u64,
}

impl Measurement<(RaplMeasurementJoules, u32)> for RaplSampler {
    fn get_measurement(&mut self, timestamp: u128) -> (RaplMeasurementJoules, u32) {
        self.update_range_map(timestamp);

        let result = self
            .range_map
            .get(&timestamp)
            .expect("No measurement found");

        // converting to joules
        (convert_to_joules(result.0.clone()), result.1)
    }

    fn get_multiple_measurements(
        &mut self,
        timestamps: &Vec<u128>,
    ) -> Vec<(RaplMeasurementJoules, u32)> {
        let mut result = Vec::new();

        // updating rangemap using the first timestamp
        self.update_range_map(timestamps[0]);

        // find measurements
        for timestamp in timestamps {
            let measurement = self.range_map.get(timestamp).expect("No measurement found");
            // converting to joules
            result.push((
                convert_to_joules(measurement.0.clone()),
                measurement.1.clone(),
            ));
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
            pkg_overflow: 0,
            last_pkg: 0,
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
            // finding overflows, by matching cpu type and checking pkg
            let measurement_clone = measurement.clone();
            match measurement_clone {
                RaplMeasurement::Intel(measurement_clone) => {
                    if self.last_pkg > measurement_clone.pkg {
                        self.pkg_overflow += 1;
                    }
                    self.last_pkg = measurement_clone.pkg;
                }
                RaplMeasurement::AMD(measurement_clone) => {
                    if self.last_pkg > measurement_clone.pkg {
                        self.pkg_overflow += 1;
                    }
                    self.last_pkg = measurement_clone.pkg;
                }
            }

            self.range_map.insert(
                time..time + self.sampling_interval as u128,
                (measurement, self.pkg_overflow),
            );
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
