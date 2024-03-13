use anyhow::Result;
use thor_lib::RaplMeasurement;

pub struct Measure {
    pub timestamp_start: u128,
    pub timestamp_end: u128,
    pub measurement: RaplMeasurement, // core
}
pub trait Measurement<T> {
    // T is the type of measurement
    fn get_measurement(&self, timestamp: u128) -> T; // hmm how this?
}

pub trait Build {
    fn build(&self, build_script: String) -> bool; // returns whether it succeded
}

pub trait StartProcess {
    fn start_process(&self, process: String) -> bool; // returns whether it succeded
}

pub trait Listener<T> {
    fn start_listening<S: StartProcess, B: Build, M: Measurement<T>>(
        &self,
        start_process: S,
        builder: B,
        measurement: M,
        port: u16,
    ) -> Result<()>;
}
