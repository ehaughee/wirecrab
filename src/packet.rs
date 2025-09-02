#[derive(Debug, Clone)]
pub struct Packet {
    pub timestamp: f64,
    pub length: u32,
    pub data: Vec<u8>,
}
