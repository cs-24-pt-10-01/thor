use crate::build::GitBuild;
use anyhow::Result;
use crossbeam::queue::SegQueue;
use serde_json;
use std::{
    collections::VecDeque,
    io::Write,
    net::Shutdown,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use thor_lib::RaplMeasurement;
use thor_shared::{ClientPacket, ConnectionType, ProcessUnderTestPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

static PROCESS_UNDER_TEST_PACKET_QUEUE: SegQueue<ProcessUnderTestPacket> = SegQueue::new();

use crate::component_def::{Build, Listener, Measurement, StartProcess};

pub struct ListenerImplem {
    pub ip: String,
    pub remote_packet_queue_cycle: u64,
}

// Needle for the end of a string (used for repoes)
const NEEDLE: &[u8] = "#".as_bytes();

// max size for repo url
const MAX_REPO_SIZE: usize = 1024;

// Delimiter for the send measurements to the clients
const MEASUREMENTS_DELIMITER: &[u8] = "end".as_bytes();

impl Listener<RaplMeasurement> for ListenerImplem {
    fn start_listening<S: StartProcess, B: Build, M: Measurement<RaplMeasurement>>(
        &self,
        start_process: S,
        builder: B,
        measurement: &mut M,
    ) -> Result<()> {
        // Creating vector of streams
        let remote_tcpstreams = Arc::new(Mutex::new(Vec::new()));

        // Create a clone of the remote_tcpstreams to pass to the thread
        let remote_tcpstreams_clone = remote_tcpstreams.clone();

        let ip = self.ip.clone();

        // Creating thread for listening
        thread::spawn(move || {
            let fut = listen(ip, remote_tcpstreams_clone);
            tokio::runtime::Runtime::new().unwrap().block_on(fut);
        });

        send_packet_to_remote_clients(
            remote_tcpstreams,
            self.remote_packet_queue_cycle,
            measurement,
        );

        Ok(())
    }
}

async fn listen(server_ip: String, remote_tcpstreams: Arc<Mutex<Vec<std::net::TcpStream>>>) {
    // Create a TCP listener
    println!("Listening on: {}", server_ip);
    let tcp_listener = TcpListener::bind(&server_ip).await.unwrap();

    // Enter the main loop
    loop {
        let (mut socket, _) = tcp_listener.accept().await.unwrap();

        // Read the connection type and handle it
        let connection_type = socket.read_u8().await.unwrap();
        if connection_type == ConnectionType::ProcessUnderTest as u8 {
            handle_process_under_test_connection(socket);
        } else {
            handle_remote_connection(remote_tcpstreams.clone(), socket).await;
        }
    }
}

fn handle_process_under_test_connection(mut socket: tokio::net::TcpStream) {
    tokio::spawn(async move {
        let mut client_buffer = vec![0; u8::MAX as usize];

        loop {
            // Read the length of the packet
            let process_under_test_packet_length = match socket.read_u8().await {
                Ok(length) => length,
                Err(_) => {
                    // If the client has disconnected, break the loop
                    break;
                }
            };

            // Read the packet itself
            if let Err(_) = socket
                .read_exact(&mut client_buffer[0..process_under_test_packet_length as usize])
                .await
            {
                // If the client has disconnected, break the loop
                break;
            }

            // Deserialize the packet using the buffer
            let process_under_test_packet: ProcessUnderTestPacket =
                bincode::deserialize(&client_buffer).unwrap();

            // Push the packet to the process under test packet queue
            PROCESS_UNDER_TEST_PACKET_QUEUE.push(process_under_test_packet);
        }
    });
}

async fn handle_remote_connection(
    remote_tcpstreams: Arc<Mutex<Vec<std::net::TcpStream>>>,
    mut socket: tokio::net::TcpStream,
) {
    let mut buf = Vec::new();
    while !buf.ends_with(NEEDLE) && buf.len() < MAX_REPO_SIZE {
        socket.read_buf(&mut buf).await.unwrap();
    }

    buf = buf[0..buf.len() - NEEDLE.len()].to_vec();

    let repo = match String::from_utf8(buf) {
        Ok(repo) => repo.trim_matches(char::from(0)).to_string(),
        Err(e) => {
            println!("Failed to parse repo: {:?}", e);
            return;
        }
    };

    println!("Received repo: {:?}", repo);

    remote_tcpstreams
        .lock()
        .unwrap()
        .push(socket.into_std().unwrap());

    if repo == "none" {
        println!("No repo provided, client assigned as observer");
        return;
    }

    // Thread for building and running process
    thread::spawn(move || {
        // build and start process
        let res = GitBuild {}.build(repo);
        match res {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to build and run repo: {:?}", e);
            }
        }

        // waiting for measurements to be sent
        while !PROCESS_UNDER_TEST_PACKET_QUEUE.is_empty() {
            thread::sleep(Duration::from_secs(1));
        }

        // Disconnecting client measurement is done
        // TODO this can break with multiple clients are connected.
        match remote_tcpstreams.lock().unwrap().pop() {
            Some(s) => {
                s.shutdown(std::net::Shutdown::Both).unwrap();
            }
            None => {
                println!("Could not disconnect client, client might already be disconnected.");
            }
        }
    });
}

fn send_packet_to_remote_clients<M: Measurement<RaplMeasurement>>(
    remote_connections: Arc<Mutex<Vec<std::net::TcpStream>>>,
    remote_packet_queue_cycle: u64,
    measurement: &mut M,
) {
    // Create duration from the config
    let duration = Duration::from_millis(remote_packet_queue_cycle);
    /*
    // Check if the duration is less than the minimum update interval
    if duration < MINIMUM_CPU_UPDATE_INTERVAL {
        panic!(
            "Remote packet queue cycle must be greater than the minimum update interval of {:?}",
            MINIMUM_CPU_UPDATE_INTERVAL
        );
    }
     */

    let mut remote_client_packets = Vec::new();

    loop {
        let mut process_under_test_packets = VecDeque::new();

        // Extract packets from processes under test initially to allow the sampler getting ahead
        while let Some(process_under_test_packet) = PROCESS_UNDER_TEST_PACKET_QUEUE.pop() {
            process_under_test_packets.push_back(process_under_test_packet);
        }

        // TODO: Consider sleeping here if the sampler is too slow, i.e. unable to find a measurement for the current packet due to time difference

        if process_under_test_packets.is_empty() {
            //keeping the sampler alive
            measurement.get_measurement(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            );
        } else {
            // Create remote client packets
            create_remote_client_packets(
                process_under_test_packets,
                measurement,
                &mut remote_client_packets,
            );

            // Get a lock on the remote connections
            let mut remote_connections_lock = remote_connections.lock().unwrap();

            if !remote_connections_lock.is_empty() && !remote_client_packets.is_empty() {
                remote_connections_lock.retain_mut(|conn| {
                    let serialized_packet = serde_json::to_vec(&remote_client_packets).unwrap();

                    // blocks if the packets is over 1 Mb
                    if serialized_packet.len() > 1000000 {
                        conn.set_nonblocking(false).unwrap();
                    }
                    // sending packets
                    match send_packet(conn, &serialized_packet) {
                        Ok(_) => {
                            conn.set_nonblocking(true).unwrap();
                            true
                        }
                        Err(_) => {
                            // Can not send to client, remove the connection
                            println!("Client did not accept the packet, removing connection");
                            conn.shutdown(Shutdown::Both).unwrap();
                            false
                        }
                    }
                });
                remote_client_packets.clear();
            }
        }

        // Sleep for the duration
        thread::sleep(duration);
    }
}

fn send_packet(
    conn: &mut std::net::TcpStream,
    serialized_packet: &Vec<u8>,
) -> Result<(), std::io::Error> {
    conn.write_all(serialized_packet)?;
    conn.write_all(MEASUREMENTS_DELIMITER)
}

fn create_remote_client_packets<M: Measurement<RaplMeasurement>>(
    mut process_under_test_packets: VecDeque<ProcessUnderTestPacket>,
    measurement: &mut M,
    remote_client_packets: &mut Vec<ClientPacket>,
) {
    let timestamps: Vec<u128> = process_under_test_packets
        .iter()
        .map(|x| x.timestamp)
        .collect();
    let measurements = measurement.get_multiple_measurements(&timestamps);

    // handling multiple packets at a time
    for x in 0..process_under_test_packets.len() {
        let rapl_measurement = measurements[x].clone();
        let process_under_test_packet = process_under_test_packets.pop_front().unwrap();
        let remote_client_packet = ClientPacket {
            process_under_test_packet,
            rapl_measurement,
        };
        remote_client_packets.push(remote_client_packet);
    }
}
