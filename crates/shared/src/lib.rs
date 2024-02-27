use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalClientPacket {
    pub id: String,
    pub process_id: u32,
    pub thread_id: usize,
    pub operation: LocalClientPacketOperation,
    pub timestamp: u128,
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
