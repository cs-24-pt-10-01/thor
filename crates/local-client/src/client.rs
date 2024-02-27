use crossbeam::queue::SegQueue;
use std::{
    io::Write,
    net::TcpStream,
    sync::Once,
    thread,
    time::{Duration, SystemTime},
};
use thor_shared::{ConnectionType, LocalClientPacket, LocalClientPacketOperation};

static RAPL_INIT: Once = Once::new();

static LOCAL_CLIENT_PACKET_QUEUE: SegQueue<LocalClientPacket> = SegQueue::new();

fn send_packet_queue() {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:6969").unwrap();

    // Signify which type of client this is (it is a local client)
    stream.write_all(&[ConnectionType::Local as u8]).unwrap();

    loop {
        while let Some(packet) = LOCAL_CLIENT_PACKET_QUEUE.pop() {
            // Serialize the packet using bincode
            let serialized = bincode::serialize(&packet).unwrap();

            // Write length and then the serialized packet
            stream.write_all(&[(serialized.len() as u8)]).unwrap();
            stream.write_all(&serialized).unwrap();
        }

        // Sleep for 2 seconds
        thread::sleep(Duration::from_secs(2));
    }
}

pub fn start_rapl(id: impl AsRef<str>) {
    // Initialize RAPL
    RAPL_INIT.call_once(|| {
        thread::spawn(send_packet_queue);
    });

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
    LOCAL_CLIENT_PACKET_QUEUE.push(packet);
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
    LOCAL_CLIENT_PACKET_QUEUE.push(packet);
}
