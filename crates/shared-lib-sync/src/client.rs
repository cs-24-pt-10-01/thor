use crossbeam::queue::SegQueue;
use std::{
    io::Write,
    net::TcpStream,
    sync::{Mutex, Once},
    thread,
    time::{Duration, SystemTime},
};
use thor_shared::{ConnectionType, LocalClientPacket, LocalClientPacketOperation};

static STREAM_INIT: Once = Once::new();

static CONNECTION: Mutex<Option<TcpStream>> = Mutex::new(None);

pub fn start_rapl(id: impl AsRef<str>) {
    // Initialize RAPL

    let packet = LocalClientPacket {
        id: id.as_ref().to_string(),
        process_id: 12345,
        thread_id: thread_id::get(),
        operation: LocalClientPacketOperation::Start,
        timestamp: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    send_packet(packet);
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let packet = LocalClientPacket {
        id: id.as_ref().to_string(),
        process_id: 12345,
        thread_id: thread_id::get(),
        operation: LocalClientPacketOperation::Stop,
        timestamp: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    send_packet(packet);
}

fn send_packet(packet: LocalClientPacket) {
    STREAM_INIT.call_once(|| {
        // making connection
        let mut connection = TcpStream::connect("127.0.0.1:6969").unwrap();

        // indicating that this is a local process
        connection
            .write_all(&[ConnectionType::Local as u8])
            .unwrap();

        *CONNECTION.lock().unwrap() = Some(connection);
    });

    let serialized = bincode::serialize(&packet).unwrap();

    let mut binding = CONNECTION.lock().unwrap();
    let stream = binding.as_mut().unwrap();

    // Write length and then the serialized packet
    stream.write_all(&[(serialized.len() as u8)]).unwrap();
    stream.write_all(&serialized).unwrap();
}
