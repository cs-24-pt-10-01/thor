use anyhow::Result;
use crate::component_def::{StartProcess, Build, Measurement, Listener};

pub struct DefList{}

impl Listener for DefList{
    fn start_listening<S: StartProcess, B: Build, M: Measurement>(
        &self,
        start_process: S,
        builder: B,
        measurement: M,
        port: u16,
    ) -> Result<()>{
        if !builder.build("".to_string()){
            print!("build failed");
        }
        if !start_process.start_process("".to_string()){
            print!("start process failed");
        }

        println!("listener not implemented");
        Ok(())
    }
}
