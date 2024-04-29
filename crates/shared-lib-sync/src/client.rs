use std::{
    io::Write,
    net::TcpStream,
    process,
    sync::{Mutex, Once},
    time::SystemTime,
};
use thor_shared::{ConnectionType, ProcessUnderTestPacket, ProcessUnderTestPacketOperation};

static STREAM_INIT: Once = Once::new();

static CONNECTION: Mutex<Option<TcpStream>> = Mutex::new(None);

pub fn start_rapl(id: impl AsRef<str>) {
    let packet = ProcessUnderTestPacket {
        id: id.as_ref().to_string(),
        process_id: process::id(),
        thread_id: thread_id::get(),
        operation: ProcessUnderTestPacketOperation::Start,
        timestamp: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    send_packet(packet);
}

pub fn stop_rapl(id: impl AsRef<str>) {
    let packet = ProcessUnderTestPacket {
        id: id.as_ref().to_string(),
        process_id: process::id(),
        thread_id: thread_id::get(),
        operation: ProcessUnderTestPacketOperation::Stop,
        timestamp: SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };

    send_packet(packet);
}

fn send_packet(packet: ProcessUnderTestPacket) {
    STREAM_INIT.call_once(|| {
        // making connection
        let mut connection = TcpStream::connect("127.0.0.1:6969").unwrap();

        // indicating that this is a process under test process
        connection
            .write_all(&[ConnectionType::ProcessUnderTest as u8])
            .unwrap();
        // TODO: Consider sending PID here to identify the process, as PID does not change.
        // Then remove it from the packet

        *CONNECTION.lock().unwrap() = Some(connection);
    });

    let serialized = bincode::serialize(&packet).unwrap();

    let mut connection_lock = CONNECTION.lock().unwrap();
    let stream = connection_lock.as_mut().unwrap();

    // Write length and then the serialized packet
    stream.write_all(&[(serialized.len() as u8)]).unwrap();
    stream.write_all(&serialized).unwrap();
}
