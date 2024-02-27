use csv::WriterBuilder;
use std::{error::Error, fs::OpenOptions};
use thor_lib::RaplMeasurement::{Intel, AMD};
use thor_shared::{ConnectionType, RemoteClientPacket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:6969").await.unwrap();

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
