

// TCP header struct
#[derive(Debug)]
pub struct TcpHeader {
    source_port: u16,
    destination_port: u16,
    sequence_number: u32,
    ack_number: u32,
    header_length: u8,
    flags: u8, // URG, ACK, PSH, RST, SYN, FIN (each 1 bit)
    window_size: u16,
}

impl TcpHeader {
    // Create a new TCP header
    pub fn new(header_bytes: &[u8]) -> Self {
        if header_bytes.len() < 16 {
            panic!("Header too short");
        }
        // Parse the header bytes into the fields of the header
        let source_port = u16::from_be_bytes(header_bytes[0..2].try_into().unwrap());
        let destination_port = u16::from_be_bytes(header_bytes[2..4].try_into().unwrap());
        let sequence_number = u32::from_be_bytes(header_bytes[4..8].try_into().unwrap());
        let ack_number = u32::from_be_bytes(header_bytes[8..12].try_into().unwrap());
        let header_length = header_bytes[12] >> 4; // get the first 4 bits
        let flags = header_bytes[13] & 0b0011_1111; // get the last 6 bits
        let window_size = u16::from_be_bytes(header_bytes[14..16].try_into().unwrap());

        TcpHeader {
            source_port,
            destination_port,
            sequence_number,
            ack_number,
            header_length,
            flags,
            window_size,
        }
    }
}