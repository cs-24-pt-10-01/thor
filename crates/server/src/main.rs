use crate::{
    build::GitBuild, component_def::Listener, listener::ListenerImplem, measurement::RaplSampler,
    start_process::StartImplem,
};
use config::Config;
use std::{fs, sync::Arc, thread::sleep};
use thor_lib::{IntelRaplRegistersJoules, RaplMeasurementJoules};
use thor_shared::{ClientPacket, ProcessUnderTestPacket, ProcessUnderTestPacketOperation};

mod build;
mod component_def;
mod config;
mod listener;
mod measurement;
mod start_process;

fn print_put_packet_len(put_packet: ProcessUnderTestPacket) {
    let serialized_put = bincode::serialize(&put_packet).unwrap();
    println!("serialized put len: {}", serialized_put.len());
}

fn print_client_packet_len(client_packet: &ClientPacket) {
    let serialized_client = bincode::serialize(&client_packet).unwrap();
    println!("serialized client len: {}", serialized_client.len());
}

fn print_client_packets_len(client_packets: &Vec<ClientPacket>) {
    let serialized_client = bincode::serialize(&client_packets).unwrap();
    println!("serialized clients len: {}", serialized_client.len());
}

fn main() {
    print_put_packet_len(ProcessUnderTestPacket {
        id: "".to_string(),
        process_id: 0,
        thread_id: 0,
        operation: ProcessUnderTestPacketOperation::Start,
        timestamp: 0,
    });

    print_put_packet_len(ProcessUnderTestPacket {
        id: "AAAAA".to_string(),
        process_id: 0,
        thread_id: 0,
        operation: ProcessUnderTestPacketOperation::Start,
        timestamp: 0,
    });
    print_put_packet_len(ProcessUnderTestPacket {
        id: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        process_id: 0,
        thread_id: 0,
        operation: ProcessUnderTestPacketOperation::Start,
        timestamp: 0,
    });

    let client_packet = ClientPacket {
        process_under_test_packet: ProcessUnderTestPacket {
            id: "".to_string(),
            process_id: 0,
            thread_id: 0,
            operation: ProcessUnderTestPacketOperation::Start,
            timestamp: 0,
        },
        rapl_measurement: RaplMeasurementJoules::Intel(IntelRaplRegistersJoules {
            pp0: 0.0,
            pp1: 0.0,
            pkg: 0.0,
            dram: 0.0,
        }),
        pkg_overflow: 0,
    };
    serde_json::to_vec(&client_packet).unwrap();
    print_client_packet_len(&client_packet);

    let mut client_packets = Vec::new();
    client_packets.push(client_packet);
    print_client_packets_len(&client_packets);

    let client_packet2 = ClientPacket {
        process_under_test_packet: ProcessUnderTestPacket {
            id: "".to_string(),
            process_id: 0,
            thread_id: 0,
            operation: ProcessUnderTestPacketOperation::Start,
            timestamp: 0,
        },
        rapl_measurement: RaplMeasurementJoules::Intel(IntelRaplRegistersJoules {
            pp0: 0.0,
            pp1: 0.0,
            pkg: 0.0,
            dram: 0.0,
        }),
        pkg_overflow: 0,
    };

    client_packets.push(client_packet2);
    print_client_packets_len(&client_packets);

    //getting config
    let config_file_data =
        fs::read_to_string("thor-server.toml").expect("Failed to read thor-server.toml");
    let config: Arc<Config> =
        Arc::new(toml::from_str(&config_file_data).expect("Failed to parse config"));

    let start = StartImplem {};

    let build = GitBuild {};

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
    listen.start_listening(start, build, &mut measure).unwrap();
}
