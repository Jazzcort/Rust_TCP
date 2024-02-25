mod util;
mod tcp_sender;
use std::any::TypeId;
use std::io::{self, BufRead, Stdin, StdinLock};
use clap::Parser;
use std::net::UdpSocket;
use std::thread::{JoinHandle, spawn};
use util::tcp_header::TcpHeader;
use util::util::*;
use tcp_sender::Sender;



#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    recv_host: String,
    recv_port: String
}


fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let port = cli.recv_port.parse::<u16>().unwrap();

    let mut sender = Sender::new(cli.recv_host, port, "127.0.0.1".to_string()).unwrap();

    dbg!(&sender);

    sender.start();


    // let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    // let mut buffer = String::new();
    // let stdin = io::stdin();
    // let mut handle = stdin.lock();

    // let data_len = handle.read_line(&mut buffer).map_err(|e| format!("{e} -> failed to read"))?;

    // let texts: Vec<String> = buffer.as_bytes().chunks(1500).map(| ch| String::from_utf8_lossy(ch).to_string()).collect();

    // let mut acks: Vec<usize> = vec![0; texts.len()];

    // for text in texts.iter() {
    //     socket.send_to(text.as_bytes() , format!("{}:{}", cli.recv_host.clone(), cli.recv_port.clone())).unwrap();
    // }

    // let mut buf: [u8; 1500] = [0; 1500];
    // loop {
    //     match socket.recv(&mut buf) {
    //         Ok(_) => {
    //             let ack = read_to_string(&buf);
    //             buf.fill(0);
    //             let ind = ack.parse::<usize>().unwrap();
    //             acks[ind] = 1;

    //             if acks.iter().sum::<usize>() == acks.len() {
    //                 break;
    //             }
    //         }
    //         Err(_) => {}
    //     }
    // }
    Ok(())
}
