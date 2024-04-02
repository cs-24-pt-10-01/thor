use csv::WriterBuilder;
use serde::Deserialize;
use serde_json;
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

// Delimiter to define an end of a packet
const END: &str = "end";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load the config file
    let config_file_data = fs::read_to_string("remote-client.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Connect to the server
    println!("Connecting to server at: {}", config.server_ip);
    let mut stream = TcpStream::connect(config.server_ip).await.unwrap();

    // Signify which type of client this is (it is a local client)
    stream
        .write_all(&[ConnectionType::Remote as u8])
        .await
        .unwrap();

    // signifying no repo
    stream.write_all(b"none").await.unwrap();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(format!("test.csv"))?;

    // Create the CSV writer
    let mut wtr = WriterBuilder::new().from_writer(file);

    let mut client_buffer = Vec::new();

    loop {
        let read_bytes = stream.read_buf(&mut client_buffer).await.unwrap();
        if read_bytes == 0 {
            continue;
        }

        if client_buffer.ends_with(END.as_bytes()) {
            let remote_client_packets: Vec<RemoteClientPacket> =
                serde_json::from_slice(&&client_buffer[..&client_buffer.len() - END.len()])
                    .unwrap();
            // Write the measurements to the CSV file
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
                        wtr.serialize((
                            remote_client_packet.local_client_packet,
                            amd_rapl_registers,
                        ))?;
                        wtr.flush()?;
                    }
                }
                wtr.flush()?;
            }
            client_buffer.clear();
        }
    }
}
