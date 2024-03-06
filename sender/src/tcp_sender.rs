use rand::prelude::*;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::format;
use std::io::{self, BufRead, Stdin, StdinLock};
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use sha2::{Sha256, Digest};
use generic_array::GenericArray;
use typenum::U32;

use crate::util::tcp_header::TcpHeader;
use crate::{read_to_string, safe_increment};

// Sender status
#[derive(Debug)]
enum Status {
    StandBy, // Waiting for stdin 
    Handshake,
    Sending, // Last SEND will send FIN packet
    Finished, // After sending 
}

// Packet struct
#[derive(Clone, Debug)]
struct Packet {
    timestamp: Instant, // time when packet is sent
    data: Vec<u8>,
    seq_num: u32,
    ack_num: u32,
    confirm_ack: u32, // Ack number supposed to be, used for retransmission
    data_len: u16, // length for data
}

// Sender struct
#[derive(Debug)]
pub struct Sender {
    remote_host: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
    status: Status,
    seq_num: u32,
    ack_num: u32,
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
    cur_buf: u16, // Length of data in flight (only data, not including header)
    pre_ack: u32, // Latest ACK that received
}

impl Sender {
    // Constructor
    pub fn new(remote_host: String, remote_port: u16, local_host: String) -> Result<Self, String> {
        // Generate a random sequence number
        let mut rng = rand::thread_rng();
        let seq_num: u32 = rng.gen();

        // Bind socket to a random port
        let socket = UdpSocket::bind(format!("{}:{}", local_host, 0))
            .map_err(|e| format!("{} -> Failed to bind to {}:{}", e, local_host, 0))?;
        // Get the local address
        let local = socket
            .local_addr()
            .map_err(|e| format!("{e} -> Failed to get local port"))?;
        // Switch to non-blocking
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
            data: VecDeque::new(),
            socket,
            rto: 800, // Initial RTO
            rtt: 400, // Initial RTT
            in_flight: VecDeque::new(),
            wnd_size: 46720,
            ssthresh: 32,
            count: 0,
            cur_wnd: 46720,
            cwnd: 4,
            cur_buf: 1,
            pre_ack: 0,
        })
    }

    // Start the sender
    pub fn start(&mut self) -> Result<(), String> {
        loop {
            eprintln!("seq# {}, ack# {}", self.seq_num, self.ack_num);

            match self.status {
                // Read from stdin
                Status::StandBy => {
                    // Read from stdin
                    eprintln!("Standby");
                    let mut buffer = String::new();
                    let stdin = io::stdin();
                    let mut handle = stdin.lock(); // ensure exclusive access to stdin
                    handle
                        .read_line(&mut buffer)
                        .map_err(|e| format!("{e} -> Failed to read stdin"))?;
                    eprintln!("{}", buffer.len());

                    self.data = buffer
                        .as_bytes() // convert string to bytes
                        .chunks(1440) // split into chunks of 1440 bytes and return an iterator
                        .map(|ch| String::from_utf8_lossy(ch).to_string()) // Turn each chunk into a string
                        .collect();
                    eprintln!("data length: {}", self.data.len());
                    self.status = Status::Handshake;
                }
                // Handshake
                Status::Handshake => {
                    eprintln!("Handshake");
                    let mut header = TcpHeader {
                        source_port: self.local_port,
                        destination_port: self.remote_port, // simulator's port
                        sequence_number: self.seq_num,
                        ack_number: self.ack_num,
                        header_length: 4, // unit of 4 bytes
                        flags: 0b0000_0010,
                        window_size: self.wnd_size,
                        hash_value: [0; 32].into(), // testing
                    };

                    // Get the hash value of the header
                    header.hash_value = header.calculate_header_hash();
                    // Prepare the packet to in flight, and send it
                    self.register_packet(header, ""); 

                    let mut buf: [u8; 1500] = [0; 1500];
                    // Get the SYN-ACK packet
                    loop {
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                // The first 16 bytes of the buffer are used to create a new TcpHeader instance.
                                let header = TcpHeader::new(&buf[..48]);
                                buf.fill(0);

                                // Check if the hash value of the header matches the hash value in the header
                                if !Self::check_hash(&header) {
                                    eprintln!("hash mismatch");
                                    // check the tcp header
                                    eprintln!("header: {:?}", header);
                                    eprintln!("sender hash: {:?}", header.hash_value);
                                    continue;
                                }

                                if header.ack_number != self.in_flight[0].confirm_ack {
                                    continue;
                                }

                                if header.flags != 18 { // ACK, SYN = 18
                                    continue;
                                }
                                // Set window size to minimum of receiver adv window and sender's adv window size
                                let adv_wnd = self.wnd_size.min(header.window_size);
                                // Set sshtresh to adv_wnd / 1440
                                self.ssthresh = adv_wnd / 1440;
                                self.cur_wnd = self.cwnd * 1440; 
                                self.in_flight.pop_front();
                                self.ack_num = safe_increment(header.sequence_number, 1);
                                // After handshake, send data
                                let mut header = TcpHeader {
                                    source_port: self.local_port,
                                    destination_port: self.remote_port,
                                    sequence_number: self.seq_num, 
                                    ack_number: self.ack_num,
                                    header_length: 4,
                                    flags: 0b0001_0000,
                                    window_size: self.wnd_size,
                                    hash_value: [0; 32].into(), // testing
                                };
                                // Get the hash value of the header
                                header.hash_value = header.calculate_header_hash();

                                self.register_packet(header, "");
                                self.status = Status::Sending; // Change status to sending
                                break;
                            }
                            Err(_) => {}
                        }

                        self.check_retransmission();
                    }
                }
                // Sending data
                Status::Sending => {
                    eprintln!("Sending");
                    while !self.in_flight.is_empty() || !self.data.is_empty() {
                        self.check_retransmission();

                        let mut buf: [u8; 1500] = [0; 1500];
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..48]);

                                // Check if the hash value of the header matches the hash value in the header
                                if !Self::check_hash(&header) {
                                    eprintln!("Sending hash mismatch");
                                    continue;
                                }

                                if header.flags != 16 { // ACK = 16
                                    eprintln!("Sending flag mismatch");
                                    continue;
                                }
                                // Adjust cwnd and ssthresh
                                if header.ack_number == self.pre_ack {
                                
                                    self.count += 1;
                                    if self.count >= 3 {
                                        Self::send_data(&self.remote_host, &self.remote_port, self.in_flight[0].data.as_slice(), &self.socket);
                                        self.cwnd = self.cwnd / 2;
                                        self.count = 0;
                                    }
                                } // if not duplicate ack
                                else {
                                    self.count = 0;

                                    if self.cwnd > self.ssthresh {
                                        self.cwnd += 2;
                                        eprintln!("greater than ssthresh");

                                    } else {
                                        self.cwnd = self.cwnd << 1; // slow start
                                    }
                                }

                                if self.cwnd >= 42 {
                                    self.cwnd = 42;
                                } else if self.cwnd < 2 {
                                    self.cwnd = 2;
                                }

                                self.cur_wnd = self.cwnd * 1440;

                                eprintln!("cwnd: {}", self.cwnd);
                                eprintln!("cur_wnd: {}", self.cur_wnd);
                                eprintln!("cur_buf: {}", self.cur_buf);
                                eprintln!("pre_ack: {}", self.pre_ack);
                                eprintln!("in flight: {}", self.in_flight.len());
                                eprintln!("ssthresh: {}", self.ssthresh);

                                // Based on the acknowledgment number in the received packet, pop the packet in the in_flight queue.
                                match Self::find_packet_index(&self.in_flight, header.ack_number) {
                                    Ok(ind) => {
                                        // oops through and removes all packets up to and including the packet that was acknowledged.
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
                                        // Updates pre_ack to the acknowledgment number from the received packet.
                                        self.pre_ack = header.ack_number;
                                    }
                                    Err(_) => {}
                                }
                                // Converts any payload data in the received packet (beyond the TCP header) to a string.
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
                            let mut header = TcpHeader {
                                source_port: self.local_port,
                                destination_port: self.remote_port,
                                sequence_number: self.seq_num,
                                ack_number: self.ack_num,
                                header_length: 4,
                                flags: 0b0001_1000,
                                window_size: self.wnd_size,
                                hash_value: [0; 32].into(), // testing
                            };

                            // Hash header and data
                            header.hash_value = header.calculate_header_data_hash(packet_data.as_bytes());

                            self.register_packet(header, &packet_data);
                            self.cur_buf += packet_data.len() as u16;
                        }
                    }
                    self.status = Status::Finished; 
                }
                // After sending all data, send last packet to tell receiver that it's finished
                Status::Finished => {
                    eprintln!("Finished");

                    let mut header = TcpHeader {
                        source_port: self.local_port,
                        destination_port: self.remote_port,
                        sequence_number: self.seq_num,
                        ack_number: self.ack_num,
                        header_length: 4,
                        flags: 0b0000_0001,
                        window_size: self.wnd_size,
                        hash_value: [0; 32].into(), // testing
                    };

                    // Get the hash value of the header
                    header.hash_value = header.calculate_header_hash();

                    self.register_packet(header, "");

                    let mut buf: [u8; 1500] = [0; 1500];
                    loop {
                        self.check_retransmission(); // Check if it's RTO
                        match self.socket.recv(&mut buf) {
                            Ok(_) => {
                                let header = TcpHeader::new(&buf[..48]);
                                // Check if the hash value of the header matches the hash value in the header
                                if !Self::check_hash(&header) {
                                    continue;
                                }

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

    // Helper function to check if the hash value of the header matches the hash value in the header
    fn check_hash(header: &TcpHeader) -> bool {
        let hash = header.calculate_header_hash();
        hash == header.hash_value
    }

    // Helper function to check if the hash value of the header and data matches the hash value in the header
    fn check_header_data_hash(header: &TcpHeader, data: &[u8]) -> bool {
        let hash = header.calculate_header_data_hash(data);
        hash == header.hash_value
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

        // Converts the TCP header into its byte representation and appends each byte to packet_data.
        for d in header.as_bytes() {
            packet_data.push(d);
        }

        // Encodes the packet's data (payload) into bytes and appends it to packet_data, while also calculating the data length.
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
            confirm_ack: safe_increment(seq_num, data_len as u32), // Ack number supposed to be, used for retransmission
            data_len, // length for data
        };

        // Adds the constructed packet to a queue (in_flight) of packets that have been sent but not yet acknowledged.
        self.in_flight.push_back(packet);


        Self::send_data(
            &self.remote_host,
            &self.remote_port,
            packet_data.as_slice(),
            &self.socket,
        );

        self.seq_num = safe_increment(seq_num, data_len as u32);
    }

    // Manage the retransmission of packets that have not been acknowledged within a certain timeout period.
    fn check_retransmission(&mut self) {
        let mut num = self.cwnd;
        let mut cnt: u16 = 0;

        // Iterates over the packets currently in flight (sent but not yet acknowledged) with mutable access. 
        for packet in self.in_flight.iter_mut() {
            if num == 0 {
                break;
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

                if num == self.cwnd {
                    self.ssthresh = self.cwnd / 2;
                }

                num -= 1;
                cnt += 1;

                if cnt == self.ssthresh {
                    break;
                }

            } else {
                break;
            }
        }

        // if num != self.cwnd {
        //     self.cwnd = 2;
        // }
    }
    // Helper function to send data
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
