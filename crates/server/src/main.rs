use crate::{component_def::Listener, listener::ListenerImplem, measurement::RaplSampler};
use config::Config;
use std::{fs, sync::Arc, thread::sleep};

mod build;
mod component_def;
mod config;
mod listener;
mod measurement;

fn main() {
    //getting config
    let config_file_data =
        fs::read_to_string("thor-server.toml").expect("Failed to read thor-server.toml");
    let config: Arc<Config> =
        Arc::new(toml::from_str(&config_file_data).expect("Failed to parse config"));

    let mut measure = RaplSampler::new(
        config.thor.max_sample_age_millis as u128,
        config.thor.sampling_interval_micros,
    );

    // waiting for Sampler to begin
    sleep(std::time::Duration::from_secs(1));

    let listen = ListenerImplem {
        ip: config.thor.server_ip.clone(),
        client_packet_queue_cycle: config.thor.client_packet_queue_cycle_millis,
    };
    listen.start_listening(&mut measure).unwrap();
}
