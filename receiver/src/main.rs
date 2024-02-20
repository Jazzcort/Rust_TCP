use std::net::UdpSocket;
use std::io::{self, Write};

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

    let mut buf: [u8; 1500] = [0; 1500];
    socket.recv(&mut buf).unwrap();   

    let mut output = io::stdout();
    output.write(&buf).unwrap();

    println!("{}", String::from_utf8_lossy(&buf));
}
