mod build;
mod component_def;
mod listener;
mod measurement;
mod start_process;

// definitions of components
use crate::component_def::{Build, Listener, Measurement, StartProcess};

// implementations of components
use crate::build::BuilderImplem;
use crate::listener::ListenerImplem;
use crate::measurement::RaplSampler;
use crate::start_process::StartImplem;

use std::thread::sleep;

fn main() {
    let start = StartImplem {};

    let build = BuilderImplem {};

    let measure = RaplSampler;
    measure
        .start_measureing(1000)
        .expect("Failed to start measuring");

    // waiting for measurement to begin
    sleep(std::time::Duration::from_secs(2));

    let listen = ListenerImplem {};
    let _ = listen.start_listening(start, build, measure, 8080);
}
