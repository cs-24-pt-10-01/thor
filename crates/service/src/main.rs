mod component_def;
mod build;
mod measurement;
mod start_process;
mod listener;

// definitions of components
use crate::component_def::{StartProcess, Build, Measurement, Listener};

// implementations of components
use crate::build::defBuild;
use crate::measurement::defMeasure;
use crate::start_process::defStart;
use crate::listener::defList;

fn main() {
    let start = defStart{};
    let build = defBuild{};
    let measure = defMeasure{};
    let listen = defList{};
    listen.start_listening(start, build, measure, 8080);
}
