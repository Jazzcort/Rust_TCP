mod util;
use std::fmt::format;
mod tcp_receiver;
use std::net::UdpSocket;

use crate::util::tcp_header::TcpHeader;
use crate::util::util::*;
use tcp_receiver::Receiver;


fn main() {

    let mut receiver = Receiver::new("127.0.0.1".to_string()).unwrap();

    receiver.start();

    // let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    // socket.set_nonblocking(true).unwrap();
    // eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

    // let mut buf: [u8; 1500] = [0; 1500];
    // let mut i = 0;
    // loop {
    //     match socket.recv(&mut buf) {
    //         Ok(_) => {
    //             let header = TcpHeader::new(&buf[..16]);
    //             dbg!(&header);
    //             let header2 = TcpHeader {
    //                 source_port: socket.local_addr().unwrap().port(),
    //                 destination_port: header.source_port,
    //                 sequence_number: 1993,
    //                 ack_number: safe_increment(header.sequence_number, 1),
    //                 header_length: 4,
    //                 flags: 18,
    //                 window_size: 6000
    //             };

    //             socket.send_to(&header2.as_bytes(), format!("{}:{}", "127.0.0.1", header.source_port)).unwrap();
    
    //             buf.fill(0);
    //         }
    //         Err(_) => {}
            
    //     }
    // }
    
    // let mut res = String::new();
    // let mut i = 0;
    // loop {
    //     match socket.recv_from(&mut buf) {
    //         Ok((_, addr)) => {
    //             let text = read_to_string(&buf);
    //             if text == "jazzcort" {
    //                 print!("{}", res);
    //             }

    //             match socket.send_to(format!("{}", i).as_bytes(), addr) {
    //                 Ok(_) => {eprintln!("Successfully sent")}
    //                 Err(_) => {eprintln!("Failed to send")}
    //             }

    //             res.push_str(text.as_str());
    //             // print!("{}", String::from_utf8_lossy(&buf));
    //             buf.fill(0);
    //             i += 1;
    //         }
    //         Err(_) => {}
    //     }
    // }

}
