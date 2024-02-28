use csv::WriterBuilder;
use serde::Deserialize;
use std::{
    error::Error,
    fs::{self, OpenOptions},
};
use thor_lib::RaplMeasurement::{Intel, AMD};
use thor_shared::{ConnectionType, RemoteClientPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[derive(Debug, Deserialize)]
struct Config {
    server_ip: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the config file
    let config_file_data = fs::read_to_string("remote-client.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Connect to the server
    let mut stream = TcpStream::connect(config.server_ip).await.unwrap();

    // Signify which type of client this is (it is a local client)
    stream
        .write_all(&[ConnectionType::Remote as u8])
        .await
        .unwrap();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("test.csv"))?;

    // Create the CSV writer
    let mut wtr = WriterBuilder::new().from_writer(file);

    let mut client_buffer = vec![0; u16::MAX as usize];

    loop {
        let packet_length = stream.read_u16().await.unwrap();

        // Read exactly packet_length bytes
        stream
            .read_exact(&mut client_buffer[..packet_length as usize])
            .await
            .unwrap();

        let remote_client_packets: Vec<RemoteClientPacket> =
            bincode::deserialize(&client_buffer).unwrap();
        println!("Remote client packet: {:?}", remote_client_packets);

        for remote_client_packet in remote_client_packets {
            match remote_client_packet.rapl_measurement {
                Intel(ref intel_rapl_registers) => {
                    wtr.serialize((
                        remote_client_packet.local_client_packet,
                        intel_rapl_registers,
                    ))?;
                    wtr.flush()?;
                }
                AMD(ref amd_rapl_registers) => {
                    wtr.serialize((remote_client_packet.local_client_packet, amd_rapl_registers))?;
                    wtr.flush()?;
                }
            }
            wtr.flush()?;
        }
    }
}
