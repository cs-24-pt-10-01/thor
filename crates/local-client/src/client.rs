use std::{io::Write, net::TcpStream, sync::Mutex};
use thor_shared::{LocalClientPacket, LocalClientPacketEnum};

// TODO: Consider multiple TCPstreams for multiple threads
static CLIENT_STREAM: Mutex<Option<TcpStream>> = Mutex::new(None);

pub fn start_rapl(id: impl AsRef<str>) {
    let packet = LocalClientPacketEnum::StartRaplPacket(LocalClientPacket {
        id: id.as_ref().to_string(),
        process_id: 12345,
        thread_id: thread_id::get(),
    });
    send_packet(packet);
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let packet = LocalClientPacketEnum::StopRaplPacket(LocalClientPacket {
        id: id.as_ref().to_string(),
        process_id: 12345,
        thread_id: thread_id::get(),
    });
    send_packet(packet);
}

fn send_packet(packet: LocalClientPacketEnum) {
    // serialize it using bincode
    let serialized = bincode::serialize(&packet).unwrap();

    // TODO: Consider a hashmap of streams, and then get the stream based on the thread id
    let mut stream_lock = CLIENT_STREAM.lock().unwrap();

    // Get the stream else initialize it
    let mut stream = match *stream_lock {
        Some(ref stream) => stream,
        None => {
            let mut stream = TcpStream::connect("127.0.0.1:6969").unwrap();
            // Send a 0 byte to indicate that this is a local client
            stream.write_all(&[0]).unwrap();

            *stream_lock = Some(stream);

            println!("CONNECTED");

            stream_lock.as_ref().unwrap()
        }
    };

    println!("{:?}", stream);

    // Print len
    println!("Sending packet of length: {}", serialized.len());

    // Write length and then the serialized packet
    stream.write_all(&[(serialized.len() as u8)]).unwrap();
    stream.write_all(&serialized).unwrap();
}
