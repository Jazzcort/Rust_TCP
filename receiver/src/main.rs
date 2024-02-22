use std::net::UdpSocket;
use std::io::{self, Write};

mod tcp_header;

use tcp_header::TcpHeader;

fn main() {

    // Example TCP header bytes
    let header_bytes: [u8; 16] = [
        0x00, 0x50, // Source port (80)
        0x01, 0xBB, // Destination port (443)
        0x00, 0x00, 0x00, 0x01, // Sequence number
        0x00, 0x00, 0x00, 0x00, // Acknowledgment number
        0x50, // Header length (5) and Reserved (0), here for simplicity
        0b0011_1111, // Flags (URG, ACK, PSH, RST, SYN, FIN)
        0x00, 0x00, // Window size (just an example)
    ];

    let tcp_header = TcpHeader::new(&header_bytes);
    println!("{:?}", tcp_header);

    // let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    // eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

    // let mut buf: [u8; 1500] = [0; 1500];
    // socket.recv(&mut buf).unwrap();   

    // let mut output = io::stdout();
    // output.write(&buf).unwrap();

    // println!("{}", String::from_utf8_lossy(&buf));
}
