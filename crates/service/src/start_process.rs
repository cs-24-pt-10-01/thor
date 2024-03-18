use crate::component_def::{Build, Listener, Measurement, StartProcess};

pub struct StartImplem {}

impl StartProcess for StartImplem {
    fn start_process(&self, process: String) -> bool {
        println!("start process not implemented");
        return false;
    }
}
