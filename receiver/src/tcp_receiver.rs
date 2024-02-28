use rand::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::time::Duration;
use std::time::Instant;
use std::thread::sleep;

use crate::util::tcp_header::TcpHeader;
use crate::{read_to_string, safe_increment};

#[derive(Debug)]
enum Status {
    StandBy,
    Handshake,
    Sending,
    Finished,
}

#[derive(Clone, Debug)]
struct Packet {
    timestamp: Instant,
    data: Vec<u8>,
    seq_num: u32,
    ack_num: u32,
    confirm_ack: u32,
    data_len: u16,
}

#[derive(Debug)]
pub struct Receiver {
    remote_host: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
    status: Status,
    seq_num: u32,
    ack_num: u32,
    expect_seq: VecDeque<u32>,
    expect_ack: VecDeque<u32>,
    init_seq: u32,
    socket: UdpSocket,
    rto: u64,
    in_flight: VecDeque<Packet>,
    wnd_size: u16,
    cur_wnd: u16,
    file: String,
    cur_buf: u16,
    cache: HashMap<u32, String>,
    seen: HashSet<u32>
}

impl Receiver {
    pub fn new(local_host: String) -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        let seq_num: u32 = rng.gen();

        let socket = UdpSocket::bind(format!("{}:{}", local_host, 0))
            .map_err(|e| format!("{} -> Failed to bind to {}:{}", e, local_host, 0))?;
        let local = socket
            .local_addr()
            .map_err(|e| format!("{e} -> Failed to get local port"))?;
        socket
            .set_nonblocking(true)
            .map_err(|e| format!("{e} -> Failed to switch to non-blocking mode"))?;

        eprintln!("Bound to port {}", socket.local_addr().unwrap().port());

        Ok(Receiver {
            remote_host: "".to_string(),
            remote_port: 0,
            local_host,
            local_port: local.port(),
            status: Status::StandBy,
            init_seq: seq_num,
            seq_num,
            ack_num: 0,
            expect_seq: VecDeque::new(),
            expect_ack: VecDeque::new(),
            socket,
            rto: 1000,
            in_flight: VecDeque::new(),
            wnd_size: 46720,
            cur_wnd: 46720,
            file: String::new(),
            cur_buf: 0,
            cache: HashMap::new(),
            seen: HashSet::new()
        })
    }

    pub fn start(&mut self) -> Result<(), String> {
        loop {
            match self.status {
                Status::StandBy => {
                    eprintln!("Standby");
                    let mut buf: [u8; 1500] = [0; 1500];
                    loop {
                        match self.socket.recv_from(&mut buf) {
                            Ok((_, addr)) => {
                                let header = TcpHeader::new(&buf[..16]);

                                // if header.destination_port != self.local_port {
                                //     continue;
                                // }

                                if header.flags != 2 {
                                    continue;
                                }

                                let a: Vec<String> = addr.to_string().split(":").map(|x| x.to_string()).collect();

                                self.remote_host = a[0].to_string();
                                self.remote_port = a[1].to_string().parse::<u16>().unwrap();
                                self.ack_num = header.sequence_number;

                                eprintln!("coming seq# {}, curtent ack# {}", header.sequence_number, self.ack_num);

                                self.send_ack("1", 0b0001_0010);

                                buf.fill(0);
                                break;
                            }
                            Err(_) => {}
                        }
                    }

                    self.status = Status::Handshake;
                }
                Status::Handshake => {
                    eprintln!("Handshake");
                    let mut buf: [u8; 1500] = [0; 1500];
                    loop {
                        // self.check_retransmission();

                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..16]);

                                if header.sequence_number != self.ack_num {
                                    continue;
                                }

                                if header.flags != 16 {
                                    continue;
                                }

                                // if header.source_port != self.remote_port {
                                //     continue;
                                // }

                                // if header.destination_port != self.local_port {
                                //     continue;
                                // }

                                // self.in_flight.pop_front();
                                
                                self.send_ack("1", 0b0001_0000);
                                self.status = Status::Sending;

                                buf.fill(0);
                                break;
                            }
                            Err(_) => {}
                        }
                    }

                    self.status = Status::Sending;
                }
                Status::Sending => {
                    eprintln!("Sending");
                    loop {

                        let mut buf: [u8; 1500] = [0; 1500];
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..16]);

                                if header.flags != 24 && header.flags != 16 && header.flags != 1 {
                                    continue;
                                }

                                // if header.source_port != self.remote_port {
                                //     continue;
                                // }

                                // if header.destination_port != self.local_port {
                                //     continue;
                                // }

                                if header.sequence_number != self.ack_num {
                                    if !self.seen.contains(&header.sequence_number) {
                                        self.seen.insert(header.sequence_number);
                                        self.cache.insert(header.sequence_number, read_to_string(&buf[16..]));
                                    }
                                    self.send_ack("", 0b0001_0000);
                                } else {

                                    if header.flags == 1 {
                                        print!("{}", &self.file);
                                        self.send_ack("1", 0b0001_0001);
                                        break;
                                    }

                                    self.seen.insert(header.sequence_number);
                                    let mut data = read_to_string(&buf[16..]);
                                    let cached_date = self.check_cache(safe_increment(self.ack_num, data.len() as u32));

                                    data.push_str(&cached_date);
                                    self.file.push_str(&data);
                                    self.send_ack(&data, 0b0001_0000);
                                }

                                buf.fill(0);
                            }
                            Err(_) => {}
                        }

                    }
                    self.status = Status::Finished;
                    eprintln!("file length: {}", self.file.len());
                    // dbg!(&self.cache);
                }
                Status::Finished => {
                    self.send_ack("1", 0b0000_0001);
                    sleep(Duration::from_millis(100));

                    // eprintln!("Finished");
                    // dbg!(&self.file);
                    // break;
                }
            }
        }
        Ok(())
    }

    fn find_packet_index(in_flight: &VecDeque<Packet>, ack_num: u32) -> Result<usize, ()> {
        for (ind, packet) in in_flight.iter().enumerate() {
            if packet.confirm_ack == ack_num {
                return Ok(ind);
            }
        }
        Err(())
    }

    fn register_cache(&mut self, ack_num: u32, data: String) {
        self.cache.insert(ack_num, data);
    }

    fn check_cache(&mut self, mut seq_num: u32) -> String {
        let mut data = String::new();

        while self.cache.contains_key(&seq_num) {
            let tmp = self.cache.get(&seq_num).unwrap();
            seq_num += tmp.len() as u32;
            data.push_str(tmp);
        }

        data
    }

    fn send_ack(&mut self, data: &str, flags: u8) {
        if flags != 1 {
            self.ack_num = safe_increment(self.ack_num, data.len() as u32);
        }
        
        let header = TcpHeader {
            source_port: self.local_port,
            destination_port: self.remote_port,
            sequence_number: self.seq_num,
            ack_number: self.ack_num,
            header_length: 4,
            flags,
            window_size: self.wnd_size
        };

        let mut bytes = header.as_bytes();
        // bytes.push(47);

        Self::send_data(&self.remote_host, &self.remote_port, &bytes, &self.socket);
        self.seq_num = safe_increment(self.seq_num, 1);
    }

    fn register_packet(&mut self, header: TcpHeader, data: &str) {
        let mut packet_data: Vec<u8> = Vec::new();
        let seq_num = header.sequence_number;
        let ack_num = header.ack_number;

        for d in header.as_bytes() {
            packet_data.push(d);
        }

        let mut data_len: u16 = 0;
        for d in data.as_bytes() {
            packet_data.push(*d);
            data_len += 1;
        }

        if data_len == 0 {
            data_len = 1;
        }

        let packet = Packet {
            timestamp: Instant::now(),
            data: packet_data.clone(),
            seq_num,
            ack_num,
            confirm_ack: safe_increment(seq_num, data_len as u32),
            data_len,
        };

        self.in_flight.push_back(packet);

        Self::send_data(
            &self.remote_host,
            &self.remote_port,
            packet_data.as_slice(),
            &self.socket,
        );

        self.seq_num = safe_increment(seq_num, data_len as u32);
    }

    fn check_retransmission(&mut self) {
        for packet in self.in_flight.iter_mut() {
            let instant = Instant::now();
            let duration = instant.duration_since(packet.timestamp.clone());

            if duration >= Duration::from_millis(self.rto) {
                Self::send_data(
                    &self.remote_host,
                    &self.remote_port,
                    packet.data.as_slice(),
                    &self.socket,
                );
                packet.timestamp = Instant::now();
            } else {
                return;
            }
        }
    }

    fn send_data(remote_host: &str, remote_port: &u16, packet_data: &[u8], socket: &UdpSocket) {
        loop {
            match socket.send_to(packet_data, format!("{}:{}", remote_host, remote_port)) {
                Ok(_) => {
                    break;
                }
                Err(e) => {
                    eprintln!("{} -> Failed to send packet at registration", e)
                }
            }
        }
    }
}
