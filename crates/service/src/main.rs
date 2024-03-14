mod build;
mod component_def;
mod listener;
mod measurement;
mod start_process;

use rangemap::RangeMap;

// definitions of components
use crate::component_def::{Build, Listener, Measurement, StartProcess};

// implementations of components
use crate::build::defBuild;
use crate::listener::DefList;
use crate::measurement::defMeasure;
use crate::start_process::defStart;

use std::thread::sleep;

fn main() {
    let start = defStart {};

    let build = defBuild {};

    let measure = defMeasure;
    measure
        .start_measureing(1000)
        .expect("Failed to start measuring");

    // waiting for measurement to begin
    sleep(std::time::Duration::from_secs(2));

    let listen = DefList {};
    listen.start_listening(start, build, measure, 8080);
}
