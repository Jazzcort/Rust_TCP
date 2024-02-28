// TCP header struct
#[derive(Debug)]
pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence_number: u32,
    pub ack_number: u32,
    pub header_length: u8,
    pub flags: u8, // URG, ACK, PSH, RST, SYN, FIN (each 1 bit)
    pub window_size: u16,
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

    // Convert the header to a byte array
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        let source_port_str = format!("{:016b}", self.source_port);
        res.push(u8::from_str_radix(&source_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&source_port_str[8..], 2).unwrap());

        let dst_port_str = format!("{:016b}", self.destination_port);
        res.push(u8::from_str_radix(&dst_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&dst_port_str[8..], 2).unwrap());

        let seq_num_str = format!("{:032b}", self.sequence_number);
        res.push(u8::from_str_radix(&seq_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[24..], 2).unwrap());

        let ack_num_str = format!("{:032b}", self.ack_number);
        res.push(u8::from_str_radix(&ack_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[24..], 2).unwrap());
        
        let head_len_str = format!("{:08b}", self.header_length << 4);
        res.push(u8::from_str_radix(&head_len_str[..], 2).unwrap());
        
        let flag_str = format!("{:08b}", self.flags);
        res.push(u8::from_str_radix(&flag_str[..], 2).unwrap());

        let wnd_size_str = format!("{:016b}", self.window_size);
        res.push(u8::from_str_radix(&wnd_size_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&wnd_size_str[8..], 2).unwrap());

        res
    }
}