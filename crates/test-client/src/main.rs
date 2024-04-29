use csv::WriterBuilder;
use serde::Deserialize;
use serde_json;
use std::{
    error::Error,
    fs::{self, OpenOptions},
};
use thor_lib::RaplMeasurement::{Intel, AMD};
use thor_shared::{ClientPacket, ConnectionType};
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
    let config_file_data = fs::read_to_string("test-client.toml")?;
    let config: Config = toml::from_str(&config_file_data)?;

    // Connect to the server
    println!("Connecting to server at: {}", config.server_ip);
    let mut stream = TcpStream::connect(config.server_ip).await.unwrap();

    // Signify which type of connection it is (it is a client connection)
    stream
        .write_all(&[ConnectionType::Client as u8])
        .await
        .unwrap();

    // signifying no repo
    stream.write_all(b"none#").await.unwrap();

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
            let client_packets: Vec<ClientPacket> =
                serde_json::from_slice(&&client_buffer[..&client_buffer.len() - END.len()])
                    .unwrap();
            // Write the measurements to the CSV file
            for client_packet in client_packets {
                match client_packet.rapl_measurement {
                    Intel(ref intel_rapl_registers) => {
                        wtr.serialize((
                            client_packet.process_under_test_packet,
                            intel_rapl_registers,
                        ))?;
                        wtr.flush()?;
                    }
                    AMD(ref amd_rapl_registers) => {
                        wtr.serialize((
                            client_packet.process_under_test_packet,
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
