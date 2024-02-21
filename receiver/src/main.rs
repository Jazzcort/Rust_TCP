use std::io::{self, Write};
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

    

    let mut buf: [u8; 1500] = [0; 1500];
    loop {
        match socket.recv(&mut buf) {
            Ok(_) => {
                print!("{}", String::from_utf8_lossy(&buf));
                buf.fill(0);
            }
            Err(_) => {}
        }
    }

    // let mut output = io::stdout();
    // output.write(&buf).unwrap();

    // println!("{}", String::from_utf8_lossy(&buf));
}
