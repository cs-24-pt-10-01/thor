mod build;
mod component_def;
mod config;
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

use config::*;
use std::{fs, sync::Arc};

fn main() {
    //getting config
    let config_file_data =
        fs::read_to_string("thor-server.toml").expect("Failed to read thor-service.toml");
    let config: Arc<Config> =
        Arc::new(toml::from_str(&config_file_data).expect("Failed to parse config"));

    let start = StartImplem {};

    let build = BuilderImplem {};

    let measure = RaplSampler {
        max_sample_age: config.thor.max_sample_age as u128,
    };
    measure
        .start_sampling(config.thor.sampling_interval_micros)
        .expect("Failed to start measuring");

    // waiting for Sampler to begin
    sleep(std::time::Duration::from_secs(1));

    let listen = ListenerImplem {
        ip: config.thor.server_ip.clone(),
        remote_packet_queue_cycle: config.thor.remote_packet_queue_cycle_millis,
    };
    let _ = listen.start_listening(start, build, measure);
}
