use anyhow::Result;
use crossbeam::queue::SegQueue;
use std::{
    collections::VecDeque,
    io::Write,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime},
};
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;
use thor_lib::RaplMeasurement;
use thor_shared::{ConnectionType, LocalClientPacket, RemoteClientPacket};
use tokio::{io::AsyncReadExt, net::TcpListener};

static LOCAL_CLIENT_PACKET_QUEUE: SegQueue<LocalClientPacket> = SegQueue::new();

use crate::component_def::{Build, Listener, Measurement, StartProcess};

pub struct ListenerImplem {
    pub ip: String,
    pub remote_packet_queue_cycle: u64,
}

impl Listener<RaplMeasurement> for ListenerImplem {
    #[tokio::main]
    async fn start_listening<S: StartProcess, B: Build, M: Measurement<RaplMeasurement>>(
        &self,
        start_process: S,
        builder: B,
        measurement: &mut M,
    ) -> Result<()> {
        // Setup the RAPL stuff queue
        let remote_tcpstreams = Arc::new(Mutex::new(Vec::new()));

        // Spawn thread for sampling
        // Create a clone of the remote_tcpstreams and the rapl_stuff_queue to pass to the thread
        let remote_tcpstreams_clone = remote_tcpstreams.clone();

        let ip = self.ip.clone();

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
        let mut local_client_packets = VecDeque::new();

        // Extract local clients initially to allow the sampler getting ahead
        while let Some(local_client_packet) = LOCAL_CLIENT_PACKET_QUEUE.pop() {
            local_client_packets.push_back(local_client_packet);
        }

        // TODO: Consider sleeping here if the sampler is too slow, i.e. unable to find a measurement for the current packet due to time difference

        // TODO handle disconnected clients

        if local_client_packets.is_empty() {
            //keeping the sampler alive
            let _ = measurement.get_measurement(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            );
        } else {
            // Create remote client packets
            create_remote_client_packets(
                local_client_packets,
                measurement,
                &mut remote_client_packets,
            );

            // Get a lock on the remote connections
            let mut remote_connections_lock = remote_connections.lock().unwrap();

            // Send the remote client packets to the remote clients if there is any connections available
            if !remote_connections_lock.is_empty() && !remote_client_packets.is_empty() {
                for conn in remote_connections_lock.iter_mut() {
                    let serialized_packet = bincode::serialize(&remote_client_packets).unwrap();
                    conn.write_all(&(serialized_packet.len() as u32).to_be_bytes())
                        .unwrap();
                    conn.write_all(&serialized_packet).unwrap();
                }
                remote_client_packets.clear();
            }
        }
        println!("packets: {}", LOCAL_CLIENT_PACKET_QUEUE.len().to_string());
        // Sleep for the duration
        thread::sleep(duration);
    }
}

fn create_remote_client_packets<M: Measurement<RaplMeasurement>>(
    mut local_client_packets: VecDeque<LocalClientPacket>,
    measurement: &mut M,
    remote_client_packets: &mut Vec<RemoteClientPacket>,
) {
    let timestamps: Vec<u128> = local_client_packets.iter().map(|x| x.timestamp).collect();
    let measurements = measurement.get_multiple_measurements(&timestamps);

    // handling multiple packets at a time
    for x in 0..local_client_packets.len() {
        let rapl_measurement = measurements[x].clone();
        let local_client_packet = local_client_packets.pop_front().unwrap();
        let remote_client_packet = RemoteClientPacket {
            local_client_packet,
            rapl_measurement,
        };
        remote_client_packets.push(remote_client_packet);
    }
    /* sequential
    while let Some(local_client_packet) = local_client_packets.pop_front() {
        // Get the RAPL measurement
        let rapl_measurement = measurement.get_measurement(local_client_packet.timestamp);

        // Construct the remote client packet
        let remote_client_packet = RemoteClientPacket {
            local_client_packet,
            rapl_measurement,
        };

        // Push the remote client packet to the remote client packets vector
        remote_client_packets.push(remote_client_packet);
    }
    */
}
