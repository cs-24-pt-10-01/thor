use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalClientPacket {
    pub id: String,
    pub process_id: u32,
    pub thread_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LocalClientPacketOperation {
    Start,
    Stop,
}

pub enum ConnectionType {
    Local = 0,
    Remote = 1,
}

pub enum LocalOperation {
    Start = 0,
    Stop = 1,
}
