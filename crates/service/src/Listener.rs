use crate::component_def::{StartProcess, Build, Measurement, Listener};

pub struct defList{}

impl Listener for defList{
    fn start_listening<S:StartProcess,B:Build,M:Measurement>(&self, start_process:S, builder:B, measurement:M, port: u16) -> bool
    {
        // start process
        if !start_process.start_process("java -jar server.jar".to_string()){
            println!("failed to start process");
            //return false;
        }
        // build
        if !builder.build("mvn clean install".to_string()){
            println!("failed to build");
            //return false;
        }
        // start listening
        println!("would be listening on port {}, if implemented(Not implemented yet)", port);

        return true;
    }
}
