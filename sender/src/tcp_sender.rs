use rand::prelude::*;
use std::collections::VecDeque;
use std::fmt::format;
use std::io::{self, BufRead, Stdin, StdinLock};
use std::net::UdpSocket;

use crate::util::tcp_header::TcpHeader;

enum Status {
    StandBy,
    Handshake,
    Sending,
    Finished,
}

pub struct Sender {
    remote_host: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
    status: Status,
    seq_num: u32,
    ack_num: u32,
    expect_seq: VecDeque<u32>,
    expect_ack: VecDeque<u32>,
    data: Vec<String>,
    init_seq: u32,
    socket: UdpSocket,
}

impl Sender {
    pub fn new(remote_host: String, remote_port: u16, local_host: String) -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        let seq_num: u32 = rng.gen();

        let socket = UdpSocket::bind(format!("{}:{}", local_host, 0)).map_err(|e| format!("{} -> Failed to bind to {}:{}", e, local_host, 0))?;
        let local = socket.local_addr().map_err(|e| format!("{e} -> Failed to get local port"))?;

        Ok(Sender {
            remote_host,
            remote_port,
            local_host,
            local_port: local.port(),
            status: Status::StandBy,
            init_seq: seq_num,
            seq_num,
            ack_num: 0,
            expect_seq: VecDeque::new(),
            expect_ack: VecDeque::new(),
            data: vec![],
            socket,
        })
    }

    pub fn start(&mut self) -> Result<(), String> {
        loop {
            match self.status {
                Status::StandBy => {
                    let mut buffer = String::new();
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    handle.read_line(&mut buffer).map_err(|e| format!("{e} -> Failed to read stdin"))?;

                    self.data = buffer.as_bytes().chunks(1500).map(| ch| String::from_utf8_lossy(ch).to_string()).collect();
                    self.status = Status::Handshake;
                }
                Status::Handshake => {
                    let header = TcpHeader {
                        source_port: self.remote_port,
                        destination_port: self.local_port,
                        sequence_number: self.seq_num,
                        ack_number: self.ack_num,
                        header_length: 5,
                        flags: 0b0000_0010,
                        window_size: 7000
                    };
                    self.socket.send_to(&header.as_bytes(), format!("{}:{}", self.remote_host, self.remote_port)).map_err(|e| format!("{e} -> Failed to send SYN packet"))?;

                    // self.socket.recv(buf)
                }
                Status::Sending => {

                }
                Status::Finished => {
                    break;
                }
            }
        }
        Ok(())
    }
}
