use crate::component_def::{Build, Listener, Measure, Measurement, StartProcess};

pub struct defMeasure {}

impl Measurement for defMeasure {
    fn get_measurement(&self, timestamp: u128) -> Measure {
        println!("measurement not implemented returning 0.0");
        Measure {
            timestamp_start: 0,
            timestamp_end: 0,
            pkg: 0.0,
            pp0: 0.0,
        }
    }
}
