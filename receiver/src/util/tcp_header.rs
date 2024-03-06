use sha2::{Sha256, Digest};
use generic_array::GenericArray;
use typenum::U32;

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
    pub hash_value: GenericArray<u8, U32>, // 32 bytes of hash value
}

// Implement the TCP header
impl TcpHeader {
    // Create a new TCP header
    pub fn new(header_bytes: &[u8]) -> Self {
        if header_bytes.len() < 48 {
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
        let hash_value = GenericArray::clone_from_slice(&header_bytes[16..48]);

        TcpHeader {
            source_port,
            destination_port,
            sequence_number,
            ack_number,
            header_length,
            flags,
            window_size,
            hash_value,
        }
    }

    // Function to calculate the hash of the header and data
    pub fn calculate_header_data_hash(&self, data: &[u8]) -> GenericArray<u8, U32> {
        let mut hasher = Sha256::new();
        hasher.update(&self.as_bytes_without_hash());
        hasher.update(data);
        let result = hasher.finalize();
        result
    }   
    
    // Function to calculate the hash value of the header
    pub fn calculate_header_hash(&self) -> GenericArray<u8, U32> {
        let mut hasher = Sha256::new();
        hasher.update(&self.as_bytes_without_hash());
        let result = hasher.finalize();
        result
    }

    // Helper method to serialize the header without the hash_value
    fn as_bytes_without_hash(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        // Get the source port as a byte and push it to the result
        let source_port_str = format!("{:016b}", self.source_port); // convert to string of 16 bits
        res.push(u8::from_str_radix(&source_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&source_port_str[8..], 2).unwrap());

        // Get the destination port as a byte and push it to the result
        let dst_port_str = format!("{:016b}", self.destination_port);
        res.push(u8::from_str_radix(&dst_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&dst_port_str[8..], 2).unwrap());

        // Get the sequence number as a byte and push it to the result
        let seq_num_str = format!("{:032b}", self.sequence_number);
        res.push(u8::from_str_radix(&seq_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[24..], 2).unwrap());

        // Get the ack number as a byte and push it to the result
        let ack_num_str = format!("{:032b}", self.ack_number);
        res.push(u8::from_str_radix(&ack_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[24..], 2).unwrap());
        
        // Get the header length as a byte and push it to the result
        let head_len_str = format!("{:08b}", self.header_length << 4);
        res.push(u8::from_str_radix(&head_len_str[..], 2).unwrap());
        
        // Get the flags as a byte and push it to the result
        let flag_str = format!("{:08b}", self.flags);
        res.push(u8::from_str_radix(&flag_str[..], 2).unwrap());

        // Get the window size as a byte and push it to the result
        let wnd_size_str = format!("{:016b}", self.window_size);
        res.push(u8::from_str_radix(&wnd_size_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&wnd_size_str[8..], 2).unwrap());

        res
    }

    // Convert the header to a byte array
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut res: Vec<u8> = vec![];
        // Get the source port as a byte and push it to the result
        let source_port_str = format!("{:016b}", self.source_port); // convert to string of 16 bits
        res.push(u8::from_str_radix(&source_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&source_port_str[8..], 2).unwrap());

        // Get the destination port as a byte and push it to the result
        let dst_port_str = format!("{:016b}", self.destination_port);
        res.push(u8::from_str_radix(&dst_port_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&dst_port_str[8..], 2).unwrap());

        // Get the sequence number as a byte and push it to the result
        let seq_num_str = format!("{:032b}", self.sequence_number);
        res.push(u8::from_str_radix(&seq_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&seq_num_str[24..], 2).unwrap());

        // Get the ack number as a byte and push it to the result
        let ack_num_str = format!("{:032b}", self.ack_number);
        res.push(u8::from_str_radix(&ack_num_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[8..16], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[16..24], 2).unwrap());
        res.push(u8::from_str_radix(&ack_num_str[24..], 2).unwrap());
        
        // Get the header length as a byte and push it to the result
        let head_len_str = format!("{:08b}", self.header_length << 4);
        res.push(u8::from_str_radix(&head_len_str[..], 2).unwrap());
        
        // Get the flags as a byte and push it to the result
        let flag_str = format!("{:08b}", self.flags);
        res.push(u8::from_str_radix(&flag_str[..], 2).unwrap());

        // We don't need this (maybe)
        let wnd_size_str = format!("{:016b}", self.window_size);
        res.push(u8::from_str_radix(&wnd_size_str[..8], 2).unwrap());
        res.push(u8::from_str_radix(&wnd_size_str[8..], 2).unwrap());

        // Get the hash value as a byte and push it to the result
        res.extend_from_slice(&self.hash_value);

        res
    }
}