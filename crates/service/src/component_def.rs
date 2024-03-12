pub struct Measure {
    pub timestamp_start: u128,
    pub timestamp_end: u128,
    pub pkg: f64,
    pub pp0: f64 // core
}



pub trait Measurement {
    fn get_measurement(&self, timestamp: u128 ) -> Measure; // hmm how this?
}

pub trait Build {
    fn build(&self, build_script: String) -> bool; // returns whether it succeded
}

pub trait StartProcess {
    fn start_process(&self, process: String) -> bool; // returns whether it succeded
}

pub trait Listener {
    fn start_listening<S:StartProcess,B:Build,M:Measurement>(&self, start_process:S, builder:B, measurement:M, port: u16) -> bool; // returns whether it succeded
}
