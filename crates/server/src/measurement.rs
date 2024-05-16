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
        println!("rello");
        for (key, value) in self.range_map.iter() {
            println!("{:?}: {:?}", key, value);
        }

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
            let measurement = self
                .range_map
                .get(&timestamp)
                .expect("No measurement found");

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
            match measurement {
                RaplMeasurement::Intel(ref intel_rapl_registers) => {
                    if self.last_pkg > intel_rapl_registers.pkg {
                        self.pkg_overflow += 1;
                    }
                    self.last_pkg = intel_rapl_registers.pkg;
                }
                RaplMeasurement::AMD(ref amd_rapl_registers) => {
                    if self.last_pkg > amd_rapl_registers.pkg {
                        self.pkg_overflow += 1;
                    }
                    self.last_pkg = amd_rapl_registers.pkg;
                }
            }

            self.range_map.insert(
                time..time + (self.sampling_interval * 1000 + 1_000_000) as u128,
                (measurement, self.pkg_overflow),
            );
        }

        //remove old measurements (max_sample_age is in milliseconds, converting to nanoseconds)
        self.range_map
            .remove(0..timestamp - self.max_sample_age * 1_000_000);
    }
}

fn rapl_sampling_thread(sampling_interval: u64) {
    // Loop and sample the RAPL data
    loop {
        // Grab the RAPL data and the timestamp, then push it to the queue
        let rapl_measurement = read_rapl_msr_registers();
        let timestamp = get_timestamp();

        SAMPLING_THREAD_DATA.push((rapl_measurement, timestamp));

        // Sleep for the sampling interval
        thread::sleep(Duration::from_micros(sampling_interval));
    }
}

fn get_timestamp() -> u128 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
