use anyhow::Result;
use crossbeam::queue::SegQueue;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use thor_lib::{read_rapl_msr_power_unit, read_rapl_msr_registers, RaplMeasurement};
use thor_shared::LocalClientPacketEnum;
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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load the config file
    let config_file_data = std::fs::read_to_string("thor-server.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Create a TCP listener
    let tcp_listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();

    // Setup the RAPL stuff queue
    let rapl_stuff_queue = Arc::new(SegQueue::new());
    let remote_tcpstreams = Arc::new(Mutex::new(Vec::new()));

    // Create a clone of the remote_tcpstreams and the rapl_stuff_queue to pass to the thread
    let remote_tcpstreams_clone = remote_tcpstreams.clone();
    let rapl_stuff_queue_clone = rapl_stuff_queue.clone();
    std::thread::spawn(move || {
        send_packet_to_remote_clients(rapl_stuff_queue_clone, remote_tcpstreams_clone, &config);
    });

    loop {
        let (mut socket, _) = tcp_listener.accept().await.unwrap();

        println!("Accepted connection");

        // Read the connection type
        let connection_type = socket.read_i8().await.unwrap();

        let rapl_stuff_queue_clone = rapl_stuff_queue.clone();

        if connection_type == 0 {
            // Local connection
            tokio::spawn(async move {
                let mut client_buffer = vec![0; 100];

                loop {
                    // TOOD: Add extra handling for getting rapl registers on "stop rapl", so first byte could be 1 then do reading immediately.
                    // this would be before reading length of packet
                    let priority_measure_rapl = 0;

                    let packet_length = socket.read_u8().await.unwrap();

                    println!("Got packet with len: {}", packet_length);

                    // Read packet_length bytes from the socket
                    socket
                        .read_exact(&mut client_buffer[0..packet_length as usize])
                        .await
                        .unwrap();

                    let packet: LocalClientPacketEnum =
                        bincode::deserialize(&client_buffer).unwrap();

                    let rapl_measurement = read_rapl_msr_registers();

                    let remote_packet = RemotePacket {
                        id: match packet {
                            LocalClientPacketEnum::StartRaplPacket(ref start_rapl_packet) => {
                                start_rapl_packet.id.clone()
                            }
                            LocalClientPacketEnum::StopRaplPacket(ref stop_rapl_packet) => {
                                stop_rapl_packet.id.clone()
                            }
                        },
                        process_id: 123,
                        thread_id: match packet {
                            LocalClientPacketEnum::StartRaplPacket(ref start_rapl_packet) => {
                                start_rapl_packet.thread_id
                            }
                            LocalClientPacketEnum::StopRaplPacket(ref stop_rapl_packet) => {
                                stop_rapl_packet.thread_id
                            }
                        },
                        rapl_measurement,
                        timestamp: 123,
                    };

                    rapl_stuff_queue_clone.push(remote_packet);

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
    loop {
        let testy = read_rapl_msr_registers();
        println!("{:?}", testy);

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
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn send_packet_to_remote_clients(
    remote_packet_queue: Arc<SegQueue<RemotePacket>>,
    remote_connections: Arc<Mutex<Vec<std::net::TcpStream>>>,
    config: &Config,
) {
    loop {
        // TODO: Get all processes and their CPU usage

        // loop over the remote packet queue, pop them and print them
        while let Some(remote_packet) = remote_packet_queue.pop() {
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
        std::thread::sleep(std::time::Duration::from_millis(
            config.thor.remote_packet_queue_cycle,
        ));
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RemotePacket {
    id: String,
    process_id: i32,
    thread_id: usize,
    rapl_measurement: RaplMeasurement,
    timestamp: u128,
}

//static RAPL_LOGS_MAP: OnceCell<DashMap<String, RaplLog>> = OnceCell::new();
//static RAPL_LOGS_QUEUE: OnceCell<SegQueue<RaplLog>> = OnceCell::new();
