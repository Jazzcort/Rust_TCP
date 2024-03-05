mod util;
use std::fmt::format;
mod tcp_receiver;
use std::net::UdpSocket;

use crate::util::tcp_header::TcpHeader;
use crate::util::util::*;
use tcp_receiver::Receiver;

use std::hash::{ Hash, Hasher};
use std::collections::hash_map::DefaultHasher;


fn main() {

    let mut receiver = Receiver::new("127.0.0.1".to_string()).unwrap();

    receiver.start();



}
