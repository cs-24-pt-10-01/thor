use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub thor: ThorConfig,
    pub amd: AmdConfig,
    pub intel: IntelConfig,
}

#[derive(Debug, Deserialize)]
pub struct AmdConfig {
    pub core: bool,
    pub pkg: bool,
}

#[derive(Debug, Deserialize)]
pub struct IntelConfig {
    pub pp0: bool,
    pub pp1: bool,
    pub pkg: bool,
    pub dram: bool,
}

#[derive(Debug, Deserialize)]
pub struct ThorConfig {
    pub remote_packet_queue_cycle: u64,
    pub sampling_interval: u64,
    pub server_ip: String,
}
