use crate::component_def::{StartProcess, Build, Measurement, Listener};

pub struct defStart{}

impl StartProcess for defStart{
    fn start_process(&self, process: String) -> bool{
        println!("start process not implemented");
        return false;
    }
}