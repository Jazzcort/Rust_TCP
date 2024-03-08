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

// Command line arguments
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    recv_host: String,
    recv_port: String
}


fn main() -> Result<(), String> {
    // Parse command line arguments
    let cli = Cli::parse();
    let port = cli.recv_port.parse::<u16>().unwrap();
    // Get the sender ready
    let mut sender = Sender::new(cli.recv_host, port, "127.0.0.1".to_string(), 65340, 4).unwrap();
    // Start the sender
    sender.start().expect("Failed to start the sender");

    Ok(())
}
