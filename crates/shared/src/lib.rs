use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalClientPacket {
    pub id: String,
    pub process_id: u32,
    pub thread_id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LocalClientPacketEnum {
    StartRaplPacket(LocalClientPacket),
    StopRaplPacket(LocalClientPacket),
}

pub enum AuthenticationType {
    Process,
    User,
}
