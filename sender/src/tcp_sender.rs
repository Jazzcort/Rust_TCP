use rand::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::format;
use std::io::{self, BufRead, Stdin, StdinLock};
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use crate::util::tcp_header::TcpHeader;
use crate::{read_to_string, safe_increment};


#[derive(Debug)]
enum Status {
    StandBy, // Waiting for stdin 
    Handshake,
    Sending, // Last step will send FIN packet
    Finished, // After sending 
}

#[derive(Clone, Debug)]
struct Packet {
    timestamp: Instant, // time when packet is sent
    data: Vec<u8>,
    seq_num: u32,
    ack_num: u32,
    confirm_ack: u32, // Ack number supposed to be, used for retransmission
    data_len: u16, // length for data
}

#[derive(Debug)]
pub struct Sender {
    remote_host: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
    status: Status,
    seq_num: u32,
    ack_num: u32,
    expect_seq: VecDeque<u32>, // not used 
    expect_ack: VecDeque<u32>, // not used 
    data: VecDeque<String>, // Data that been segmented
    init_seq: u32, // not used 
    socket: UdpSocket,
    rto: u64, // 2 * RTT
    rtt: u64,
    in_flight: VecDeque<Packet>, // Packets that are in flight
    wnd_size: u16, // Initial window size
    cur_wnd: u16, // Current window size
    ssthresh: u16,
    cwnd: u16, // Congestion window size
    count: u8, // For duplicate ack
    file: String, // not used 
    cur_buf: u16, // Length of data in flight (only bytes, not including header)
    pre_ack: u32,
}

impl Sender {
    pub fn new(remote_host: String, remote_port: u16, local_host: String) -> Result<Self, String> {
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
            data: VecDeque::new(),
            socket,
            rto: 800,
            rtt: 400,
            in_flight: VecDeque::new(),
            wnd_size: 46720,
            ssthresh: 32,
            count: 0,
            cur_wnd: 46720,
            cwnd: 4,
            file: String::new(),
            cur_buf: 1,
            pre_ack: 0,
        })
    }

    pub fn start(&mut self) -> Result<(), String> {
        loop {
            eprintln!("seq# {}, ack# {}", self.seq_num, self.ack_num);

            match self.status {
                Status::StandBy => {
                    // Read from stdin
                    eprintln!("Standby");
                    let mut buffer = String::new();
                    let stdin = io::stdin();
                    let mut handle = stdin.lock();
                    handle
                        .read_line(&mut buffer)
                        .map_err(|e| format!("{e} -> Failed to read stdin"))?;
                    eprintln!("{}", buffer.len());

                    self.data = buffer
                        .as_bytes()
                        .chunks(1460)
                        .map(|ch| String::from_utf8_lossy(ch).to_string())
                        .collect();
                    eprintln!("data length: {}", self.data.len());
                    self.status = Status::Handshake;
                }
                Status::Handshake => {
                    eprintln!("Handshake");
                    let header = TcpHeader {
                        source_port: self.local_port,
                        destination_port: self.remote_port,
                        sequence_number: self.seq_num,
                        ack_number: self.ack_num,
                        header_length: 4,
                        flags: 0b0000_0010,
                        window_size: self.wnd_size,
                    };
                    
                    self.register_packet(header, "");

                    let mut buf: [u8; 1500] = [0; 1500];
                    loop {
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..16]);
                                buf.fill(0);

                                if header.ack_number != self.in_flight[0].confirm_ack {
                                    continue;
                                }

                                if header.flags != 18 { // ACK, SYN = 18
                                    continue;
                                }

                                let adv_wnd = self.wnd_size.min(header.window_size);
                                self.ssthresh = adv_wnd / 1460;
                                self.cur_wnd = self.cwnd * 1460;
                                self.in_flight.pop_front();
                                self.ack_num = safe_increment(header.sequence_number, 1);
                                // After handshake, send data
                                let header = TcpHeader {
                                    source_port: self.local_port,
                                    destination_port: self.remote_port,
                                    sequence_number: self.seq_num,
                                    ack_number: self.ack_num,
                                    header_length: 4,
                                    flags: 0b0001_0000,
                                    window_size: self.wnd_size,
                                };

                                self.register_packet(header, "");
                                self.status = Status::Sending; // Change status to sending
                                break;
                            }
                            Err(_) => {}
                        }

                        self.check_retransmission();
                    }
                }
                Status::Sending => {
                    eprintln!("Sending");
                    while !self.in_flight.is_empty() || !self.data.is_empty() {
                        self.check_retransmission();

                        // eprintln!("inflight: {}, data: {}", self.in_flight.len(), self.data.len());

                        let mut buf: [u8; 1500] = [0; 1500];
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..16]);

                                if header.flags != 16 {
                                    continue;
                                }

                                if header.ack_number == self.pre_ack {
                                
                                    self.count += 1;
                                    if self.count >= 3 {
                                        Self::send_data(&self.remote_host, &self.remote_port, self.in_flight[0].data.as_slice(), &self.socket);
                                        self.cwnd = self.cwnd / 2;
                                        self.count = 0;
                                    }
                                } else {
                                    self.count = 0;

                                    if self.cur_wnd > self.ssthresh {
                                        self.cwnd += 1;
                                    } else {
                                        self.cwnd = self.cwnd << 1;
                                    }
                                }

                                if self.cwnd >= 42 {
                                    self.cwnd = 42;
                                } else if self.cwnd < 2 {
                                    self.cwnd = 2;
                                }

                                self.cur_wnd = self.cwnd * 1460;

                                eprintln!("cwnd: {}", self.cwnd);
                                eprintln!("cur_wnd: {}", self.cur_wnd);
                                eprintln!("cur_buf: {}", self.cur_buf);
                                eprintln!("pre_ack: {}", self.pre_ack);
                                eprintln!("in flight: {}", self.in_flight.len());

                                match Self::find_packet_index(&self.in_flight, header.ack_number) {
                                    Ok(ind) => {
                                        for i in 0..=ind {
                                            let packet = self.in_flight.pop_front().unwrap();
                                            self.cur_buf -= packet.data_len;

                                            // Calculate RTT
                                            if i == ind {
                                                let cur_time = Instant::now();
                                                let rtt = cur_time.duration_since(packet.timestamp).as_millis();
                                                eprintln!("rtt: {}ms", rtt);
                                                self.update_rto(rtt);
                                            }
                                        }

                                        self.pre_ack = header.ack_number;
                                    }
                                    Err(_) => {}
                                }

                                let fragment = read_to_string(&buf[16..]);
                                self.ack_num = safe_increment(self.ack_num, fragment.len() as u32);

                                buf.fill(0);
                            }
                            Err(_) => {}
                        }

                        // Send data if there is enough space in sliding window
                        while !self.data.is_empty()
                            && self.cur_wnd > self.cur_buf
                            && (self.cur_wnd - self.cur_buf) > self.data[0].len() as u16
                        {
                            let packet_data = self.data.pop_front().unwrap();
                            let header = TcpHeader {
                                source_port: self.local_port,
                                destination_port: self.remote_port,
                                sequence_number: self.seq_num,
                                ack_number: self.ack_num,
                                header_length: 4,
                                flags: 0b0001_1000,
                                window_size: self.wnd_size,
                            };

                            self.register_packet(header, &packet_data);
                            self.cur_buf += packet_data.len() as u16;
                        }
                    }
                    self.status = Status::Finished; 
                }
                Status::Finished => {
                    eprintln!("Finished");

                    let header = TcpHeader {
                        source_port: self.local_port,
                        destination_port: self.remote_port,
                        sequence_number: self.seq_num,
                        ack_number: self.ack_num,
                        header_length: 4,
                        flags: 0b0000_0001,
                        window_size: self.wnd_size,
                    };

                    self.register_packet(header, "");

                    let mut buf: [u8; 1500] = [0; 1500];
                    loop {
                        self.check_retransmission();
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..16]);

                                if header.ack_number == self.in_flight[0].confirm_ack {
                                    break;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    eprintln!("rto: {}ms", self.rto);
                    // Self::send_data(&self.remote_host, &self.remote_port, &header.as_bytes(), &self.socket);

                    break;
                }
            }
        }

        // sleep(Duration::from_millis(800));
        Ok(())
    }

    // Update RTO using RTT
    fn update_rto(&mut self, rtt: u128) {
        self.rtt = (self.rtt * 85 / 100) + (rtt * 15 / 100) as u64;
        if self.rtt < 5 {
            self.rtt = 5;
        } else if self.rtt > 900 {
            self.rtt = 900;
        }
        self.rto = self.rtt * 2;
    }

    // Find the index of the packet with the given ack number
    fn find_packet_index(in_flight: &VecDeque<Packet>, ack_num: u32) -> Result<usize, ()> {
        for (ind, packet) in in_flight.iter().enumerate() {
            if packet.confirm_ack == ack_num {
                return Ok(ind);
            }
        }
        Err(())
    }

    // Prepare and send a packet
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
        let mut num = self.cwnd;

        for packet in self.in_flight.iter_mut() {
            if num == 0 {
                return;
            }

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
                eprintln!(
                    "#{} resent: {}, since last sent: {}, ret: {}",
                    num,
                    packet.confirm_ack,
                    duration.as_millis(),
                    self.rto
                );
                num -= 1;
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
