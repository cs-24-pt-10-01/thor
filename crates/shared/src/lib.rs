use serde::{Deserialize, Serialize};
use thor_lib::RaplMeasurement;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessUnderTestPacket {
    pub id: String,
    pub process_id: u32,
    pub thread_id: usize,
    pub operation: ProcessUnderTestPacketOperation,
    pub timestamp: u128,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProcessUnderTestPacketOperation {
    Start,
    Stop,
}

pub enum ConnectionType {
    ProcessUnderTest = 0,
    Remote = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteClientPacket {
    pub process_under_test_packet: ProcessUnderTestPacket,
    pub rapl_measurement: RaplMeasurement,
}
