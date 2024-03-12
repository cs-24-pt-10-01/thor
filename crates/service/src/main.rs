mod build;
mod component_def;
mod listener;
mod measurement;
mod start_process;

// definitions of components
use crate::component_def::{Build, Listener, Measurement, StartProcess};

// implementations of components
use crate::build::defBuild;
use crate::listener::DefList;
use crate::measurement::defMeasure;
use crate::start_process::defStart;

fn main() {
    let start = defStart {};
    let build = defBuild {};
    let measure = defMeasure {};
    let listen = DefList {};
    listen.start_listening(start, build, measure, 8080);
}
