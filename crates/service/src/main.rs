use anyhow::Result;
use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::{ProcessRefreshKind, System, MINIMUM_CPU_UPDATE_INTERVAL};
use thor_lib::{read_rapl_msr_power_unit, read_rapl_msr_registers, RaplMeasurement};
use thor_shared::{ConnectionType, LocalClientPacket, LocalClientPacketOperation};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

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
}

#[derive(Debug, Serialize, Deserialize)]
struct RemotePacket {
    local_client_packet: LocalClientPacket,
    local_client_packet_operation: LocalClientPacketOperation,
    rapl_measurement: RaplMeasurement,
    timestamp: u128,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load the config file
    let config_file_data = fs::read_to_string("thor-service.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Create a TCP listener
    let tcp_listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();

    // Setup the RAPL stuff queue
    let rapl_stuff_queue = Arc::new(SegQueue::new());
    let remote_tcpstreams = Arc::new(Mutex::new(Vec::new()));

    // Create a clone of the remote_tcpstreams and the rapl_stuff_queue to pass to the thread
    let remote_tcpstreams_clone = remote_tcpstreams.clone();
    let rapl_stuff_queue_clone = rapl_stuff_queue.clone();
    thread::spawn(move || {
        send_packet_to_remote_clients(rapl_stuff_queue_clone, remote_tcpstreams_clone, &config);
    });

    loop {
        let (mut socket, _) = tcp_listener.accept().await.unwrap();

        println!("Accepted connection");

        // Read the connection type
        let connection_type = socket.read_u8().await.unwrap();

        let rapl_stuff_queue_clone = rapl_stuff_queue.clone();

        if connection_type == ConnectionType::Local as u8 {
            // Local connection

            tokio::spawn(async move {
                let mut client_buffer = vec![0; 100];

                loop {
                    let start_or_stop = socket.read_u8().await.unwrap();

                    if start_or_stop == 123 as u8 {
                        handle_start_rapl_packet(
                            &mut socket,
                            &mut client_buffer,
                            &rapl_stuff_queue_clone,
                            LocalClientPacketOperation::Start,
                        )
                        .await
                    } else {
                        handle_stop_rapl_packet(
                            &mut socket,
                            &mut client_buffer,
                            &rapl_stuff_queue_clone,
                            LocalClientPacketOperation::Stop,
                        )
                        .await
                    }

                    /*let n = match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    if let Err(e) = socket.write_all(&buf[0..n]).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
                        return;
                    }*/
                }
            });
        } else {
            // Remote connection

            // Convert to a std::net::TcpStream and push it to the remote_tcpstreams
            remote_tcpstreams
                .lock()
                .unwrap()
                .push(socket.into_std().unwrap());
        }
    }

    //unsafe { rapl_string_test(func_cstring.as_ptr()) };

    //start_rapl_iter();

    // Connect to the RAPL library
    // Start tcp client and then connect
    //let mut stream = TcpStream::connect("127.0.0.1:80").unwrap();

    //stream.set_nodelay(true).unwrap();
    //stream.set_nonblocking(true).unwrap();

    // Get the data sent from the RAPL library

    // Loop as designed for macrobenchmarks
    //loop {
    //let testy = read_rapl_msr_registers();
    //println!("{:?}", testy);

    /*
    let mut data = [0; 100];
    println!("Reading data from RAPL library... 1");
    stream.read_exact(&mut data).unwrap();
    println!("Data length {}", data.len());
    println!("Data {:?}", data);
    println!("Finished reading data from RAPL library... 1");
    */

    //let output: OutputData = bincode::serde::decode_from_std_read(&mut stream, CONFIG).unwrap();
    //println!("{:?}", output);
    //println!("{:?}", output);

    // Sleep for 10 milliseconds
    //std::thread::sleep(std::time::Duration::from_millis(10));
    //}
}

fn get_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

async fn handle_start_rapl_packet(
    socket: &mut TcpStream,
    client_buffer: &mut Vec<u8>,
    rapl_stuff_queue_clone: &Arc<
        SegQueue<(
            RaplMeasurement,
            u128,
            LocalClientPacket,
            LocalClientPacketOperation,
        )>,
    >,
    local_client_packet_operation: LocalClientPacketOperation,
) {
    // Get the local client packet
    let local_client_packet = get_dat_local_client_packet_yo(socket, client_buffer).await;

    // Get the timestamp as milliseconds
    let timestamp = get_timestamp_millis();

    // Read the RAPL registers at the end to prioritize energy measurements
    let rapl_measurement = read_rapl_msr_registers();

    // Push the data to the rapl_stuff_queue
    rapl_stuff_queue_clone.push((
        rapl_measurement,
        timestamp,
        local_client_packet,
        local_client_packet_operation,
    ));
}

async fn handle_stop_rapl_packet(
    socket: &mut TcpStream,
    client_buffer: &mut Vec<u8>,
    rapl_stuff_queue_clone: &Arc<
        SegQueue<(
            RaplMeasurement,
            u128,
            LocalClientPacket,
            LocalClientPacketOperation,
        )>,
    >,
    local_client_packet_operation: LocalClientPacketOperation,
) {
    // Read the RAPL registers at the start to prioritize energy measurements
    let rapl_measurement = read_rapl_msr_registers();

    // Get the timestamp as milliseconds
    let timestamp = get_timestamp_millis();

    // Get the local client packet
    let local_client_packet = get_dat_local_client_packet_yo(socket, client_buffer).await;

    // Push the data to the rapl_stuff_queue
    rapl_stuff_queue_clone.push((
        rapl_measurement,
        timestamp,
        local_client_packet,
        local_client_packet_operation,
    ));
}

async fn get_dat_local_client_packet_yo(
    socket: &mut TcpStream,
    client_buffer: &mut Vec<u8>,
) -> LocalClientPacket {
    let packet_length = socket.read_u8().await.unwrap();

    println!("Got packet with len: {}", packet_length);

    // Read packet_length bytes from the socket
    socket
        .read_exact(&mut client_buffer[0..packet_length as usize])
        .await
        .unwrap();

    bincode::deserialize(&client_buffer).unwrap()
}

fn send_packet_to_remote_clients(
    remote_packet_queue: Arc<
        SegQueue<(
            RaplMeasurement,
            u128,
            LocalClientPacket,
            LocalClientPacketOperation,
        )>,
    >,
    remote_connections: Arc<Mutex<Vec<std::net::TcpStream>>>,
    config: &Config,
) {
    // Create duration from the config
    let duration = Duration::from_millis(config.thor.remote_packet_queue_cycle);

    // Check if the duration is less than the minimum update interval
    if duration < MINIMUM_CPU_UPDATE_INTERVAL {
        panic!(
            "Remote packet queue cycle must be greater than the minimum update interval of {:?}",
            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL
        );
    }

    // Create a system and refresh it
    // TODO: Maybe move this into the main function initially,
    // and then pass it to this function, to prevent receiving packets before it is ready
    let mut sys = System::new_all();
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
    }
    return;

    loop {
        // TODO: Get all processes and their CPU usage using the sysinfo crate

        // Loop over the remote packet queue, pop them and print them
        while let Some((
            rapl_measurement,
            timestamp,
            local_client_packet,
            local_client_packet_operation,
        )) = remote_packet_queue.pop()
        {
            // Create the remote packet using the tuple
            let remote_packet = RemotePacket {
                local_client_packet,
                local_client_packet_operation,
                rapl_measurement,
                timestamp,
            };
            println!("Remote packet: {:?}", remote_packet);

            // Lock the remote connections
            let mut remote_connections = remote_connections.lock().unwrap();

            // Loop over all the remote connections and send the remote packet
            for remote_connection in remote_connections.iter_mut() {
                let remote_packet_serialized = bincode::serialize(&remote_packet).unwrap();

                // Send the remote packet
                remote_connection
                    .write_all(&remote_packet_serialized)
                    .unwrap();

                // TODO: Handle the case where the remote connection is dead
                /*remote_connections.retain(|mut remote_connection| {
                    match remote_connection.write_all(&remote_packet_serialized) {
                        Ok(_) => true,
                        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                            println!("Connection closed");
                            false
                        }
                        Err(_) => true,
                    }
                });*/
            }
        }

        // Sleep until the next cycle
        thread::sleep(Duration::from_millis(config.thor.remote_packet_queue_cycle));
    }
}

//static RAPL_LOGS_MAP: OnceCell<DashMap<String, RaplLog>> = OnceCell::new();
//static RAPL_LOGS_QUEUE: OnceCell<SegQueue<RaplLog>> = OnceCell::new();
