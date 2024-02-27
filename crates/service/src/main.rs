use anyhow::Result;
use crossbeam::queue::SegQueue;
use rangemap::RangeMap;
use serde::Deserialize;
use std::{
    collections::VecDeque,
    fs,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use thor_lib::{read_rapl_msr_registers, RaplMeasurement};
use thor_shared::{ConnectionType, LocalClientPacket, RemoteClientPacket};
use tokio::{io::AsyncReadExt, net::TcpListener};

//pub const CONFIG: bincode::config::Configuration = bincode::config::standard();

#[derive(Debug, Deserialize)]
struct Config {
    thor: ThorConfig,
    amd: AmdConfig,
    intel: IntelConfig,
}

#[derive(Debug, Deserialize)]
struct AmdConfig {
    core: bool,
    pkg: bool,
}

#[derive(Debug, Deserialize)]
struct IntelConfig {
    pp0: bool,
    pp1: bool,
    pkg: bool,
    dram: bool,
}

#[derive(Debug, Deserialize)]
struct ThorConfig {
    remote_packet_queue_cycle: u64,
    sampling_interval: u64,
}

static LOCAL_CLIENT_PACKET_QUEUE: SegQueue<LocalClientPacket> = SegQueue::new();
static SAMPLING_THREAD_DATA: SegQueue<(RaplMeasurement, u128)> = SegQueue::new();

#[tokio::main]
async fn main() -> Result<()> {
    // Load the config file
    let config_file_data = fs::read_to_string("thor-service.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Setup the RAPL stuff queue
    let remote_tcpstreams = Arc::new(Mutex::new(Vec::new()));

    // Spawn thread for sampling
    thread::spawn(move || rapl_sampling_thread(config.thor.sampling_interval));

    // Create a clone of the remote_tcpstreams and the rapl_stuff_queue to pass to the thread
    let remote_tcpstreams_clone = remote_tcpstreams.clone();
    thread::spawn(move || {
        send_packet_to_remote_clients(remote_tcpstreams_clone, &config);
    });

    // Create a TCP listener
    let tcp_listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();

    // Enter the main loop
    loop {
        let (mut socket, _) = tcp_listener.accept().await.unwrap();

        // Read the connection type and handle it
        let connection_type = socket.read_u8().await.unwrap();
        if connection_type == ConnectionType::Local as u8 {
            handle_local_connection(socket);
        } else {
            handle_remote_connection(remote_tcpstreams.clone(), socket);
        }
    }
}

fn handle_local_connection(mut socket: tokio::net::TcpStream) {
    tokio::spawn(async move {
        let mut client_buffer = vec![0; u8::MAX as usize];

        loop {
            // Read the length of the packet
            let local_client_packet_length = match socket.read_u8().await {
                Ok(length) => length,
                Err(_) => {
                    // If the client has disconnected, break the loop
                    break;
                }
            };

            // Read the packet itself
            if let Err(_) = socket
                .read_exact(&mut client_buffer[0..local_client_packet_length as usize])
                .await
            {
                // If the client has disconnected, break the loop
                break;
            }

            // Deserialize the packet using the buffer
            let local_client_packet: LocalClientPacket =
                bincode::deserialize(&client_buffer).unwrap();

            // Push the packet to the local client packet queue
            LOCAL_CLIENT_PACKET_QUEUE.push(local_client_packet);
        }
    });
}

fn handle_remote_connection(
    remote_tcpstreams: Arc<Mutex<Vec<std::net::TcpStream>>>,
    socket: tokio::net::TcpStream,
) {
    remote_tcpstreams
        .lock()
        .unwrap()
        .push(socket.into_std().unwrap());
}

fn rapl_sampling_thread(sampling_interval: u64) {
    // Loop and sample the RAPL data
    loop {
        // Grab the RAPL data and the timestamp, then push it to the queue
        let rapl_measurement = read_rapl_msr_registers();
        let timestamp = get_timestamp_millis();
        SAMPLING_THREAD_DATA.push((rapl_measurement, timestamp));

        // Sleep for the sampling interval
        thread::sleep(Duration::from_micros(sampling_interval));
    }
}

fn get_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn send_packet_to_remote_clients(
    remote_connections: Arc<Mutex<Vec<std::net::TcpStream>>>,
    config: &Config,
) {
    // Create duration from the config
    let duration = Duration::from_millis(config.thor.remote_packet_queue_cycle);

    // Check if the duration is less than the minimum update interval
    if duration < MINIMUM_CPU_UPDATE_INTERVAL {
        panic!(
            "Remote packet queue cycle must be greater than the minimum update interval of {:?}",
            MINIMUM_CPU_UPDATE_INTERVAL
        );
    }

    let mut remote_client_packets = Vec::new();
    let mut rangemap = RangeMap::new();

    loop {
        let mut local_client_packets = VecDeque::new();

        // Extract local clients initially to allow the sampler getting ahead
        while let Some(local_client_packet) = LOCAL_CLIENT_PACKET_QUEUE.pop() {
            local_client_packets.push_back(local_client_packet);
        }

        // TODO: Consider sleeping here if the sampler is too slow, i.e. unable to find a measurement for the current packet due to time difference

        // Populate the rangemap with sampling data
        // Get the initial RAPL measurement and timestamp
        if let Some((mut initial_rapl_measurement, mut initial_timestamp)) =
            SAMPLING_THREAD_DATA.pop()
        {
            // Iterate over the RAPL measurements and timestamps
            while let Some((rapl_measurement, timestamp)) = SAMPLING_THREAD_DATA.pop() {
                // Check if the initial RAPL measurement is different from the current one,
                // and the initial timestamp is different from the current one (required for the rangemap to work properly)
                if initial_rapl_measurement != rapl_measurement && initial_timestamp != timestamp {
                    // Insert the range into the rangemap
                    rangemap.insert(initial_timestamp..timestamp, initial_rapl_measurement);

                    // Update the initial RAPL measurement and timestamp
                    initial_rapl_measurement = rapl_measurement;
                    initial_timestamp = timestamp;
                }
            }
        }

        while let Some(local_client_packet) = local_client_packets.pop_front() {
            // Get the RAPL measurement and timestamp from the rangemap
            let rapl_measurement = rangemap
                .get(&local_client_packet.timestamp)
                .expect("No RAPL measurement found for timestamp");

            // Construct the remote client packet
            let remote_client_packet = RemoteClientPacket {
                local_client_packet,
                rapl_measurement: rapl_measurement.clone(),
            };

            println!(
                "Constructed remote client packet: {:?}",
                remote_client_packet
            );

            // Push the remote client packet to the remote client packets vector
            remote_client_packets.push(remote_client_packet);
        }

        let mut remote_connections_lock = remote_connections.lock().unwrap();

        // Send the remote client packets to the remote clients if there is any connections available
        if !remote_connections_lock.is_empty() && !remote_client_packets.is_empty() {
            for conn in remote_connections_lock.iter_mut() {
                let serialized_packet = bincode::serialize(&remote_client_packets).unwrap();
                conn.write_all(&(serialized_packet.len() as u16).to_be_bytes())
                    .unwrap();
                conn.write_all(&serialized_packet).unwrap();
            }
            remote_client_packets.clear();
        }

        // Remove rangemap measurements from 5 to 10 seconds ago
        rangemap.remove((get_timestamp_millis() - 10000)..get_timestamp_millis() - 5000);

        // Sleep for the duration
        thread::sleep(duration);
    }

    // TODO: Consider handling for process usage

    // Create a system and refresh it
    // TODO: Maybe move this into the main function initially,
    // and then pass it to this function, to prevent receiving packets before it is ready
    /*let mut sys = System::new_all();
    sys.refresh_all();

    thread::sleep(Duration::from_secs(5));
    //std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

    for i in 0..5 {
        sys.refresh_processes_specifics(ProcessRefreshKind::everything().with_cpu());

        // Print all proceeses and their CPU usage
        for (pid, process) in sys.processes() {
            if process.cpu_usage() > 0.0 {
                println!(
                    "Iteration: {}, name: {}, exe: {:?}, pid: {}, cpu usage: {:?}, memory: {}, status: {:?}",
                    i,
                    process.name(),
                    process.exe(),
                    process.pid(),
                    process.cpu_usage(),
                    process.memory(),
                    process.status(),
                );
            }
        }

        // Print status of the WoW Classic process
        for process in sys.processes_by_exact_name("WowClassic.exe") {
            println!(
                "WoW Classic process: name: {}, exe: {:?}, pid: {}, cpu usage: {:?}, memory: {}, status: {:?}",
                process.name(),
                process.exe(),
                process.pid(),
                process.cpu_usage(),
                process.memory(),
                process.status(),
            );
        }

        // Sleep for the minimum CPU update interval
        thread::sleep(Duration::from_secs(10));
    }*/
}
