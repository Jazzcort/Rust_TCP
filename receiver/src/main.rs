use std::io::{self, Write};
use std::net::UdpSocket;

fn read_to_string(buf: &[u8]) -> String {
    for ind in 0..buf.len() {
        if buf[ind] == 0 {
            return String::from_utf8_lossy(&buf[..ind]).to_string();
        }
    }
    String::from_utf8_lossy(&buf).to_string()
}

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

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

    let mut buf: [u8; 1500] = [0; 1500];
    let mut res = String::new();
    let mut i = 0;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((_, addr)) => {
                let text = read_to_string(&buf);
                if text == "jazzcort" {
                    print!("{}", res);
                }

                match socket.send_to(format!("{}", i).as_bytes(), addr) {
                    Ok(_) => {eprintln!("Successfully sent")}
                    Err(_) => {eprintln!("Failed to send")}
                }

                res.push_str(text.as_str());
                // print!("{}", String::from_utf8_lossy(&buf));
                buf.fill(0);
                i += 1;
            }
            Err(_) => {}
        }
    }


    // let mut output = io::stdout();
    // output.write(&buf).unwrap();

    // println!("{}", String::from_utf8_lossy(&buf));
}
