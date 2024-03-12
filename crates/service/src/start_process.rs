use crate::component_def::{Build, Listener, Measurement, StartProcess};

pub struct defStart {}

impl StartProcess for defStart {
    fn start_process(&self, process: String) -> bool {
        println!("start process not implemented");
        return false;
    }
}
