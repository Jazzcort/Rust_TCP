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

// use sha2::{Sha256, Digest};
// use generic_array::GenericArray;
// use typenum::U32;

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

    sender.start();


    

    Ok(())
}
