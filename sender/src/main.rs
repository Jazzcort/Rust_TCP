mod util;
mod tcp_sender;
use std::any::TypeId;
use std::io::{self, BufRead, Stdin, StdinLock};
use clap::Parser;
use std::net::UdpSocket;
use std::thread::{JoinHandle, spawn};
use util::tcp_header::TcpHeader;
use util::util::*;



#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    recv_host: String,
    recv_port: String
}

// trait ReadData {
//     fn read_date(&self, buf: &mut String) -> Result<(), String>;
// }

// impl ReadData for UdpSocket {
//     fn read_date(&self, buf:&mut String) -> Result<(), String> {
//         let mut buffer: [u8; 1500] = [0; 1500];
//         match self.recv_from(&mut buffer) {
//             Ok(_) => {
//                 buf.push_str(read_to_string(&mut buffer).as_str());
//                 Ok(())
//             }
//             Err(_) => {Err(format!("No Data"))}
//         }
//     }
// }

// impl ReadData for Stdin {
//     fn read_date(&self,buf:&mut String) -> Result<(), String> {
//         match self.read_line(buf) {
//             Ok(_) => {
//                 Ok(())
//             }
//             Err(_) => {Err(format!("No Data"))} 
//         }
//     }
// }

// fn read_to_string(buf: &[u8]) -> String {
//     for ind in 0..buf.len() {
//         if buf[ind] == 0 {
//             return String::from_utf8_lossy(&buf[..ind]).to_string();
//         }
//     }
//     String::from_utf8_lossy(&buf).to_string()
// }

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let mut waiting = false;

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // let mut tmp: Vec<Box<dyn ReadData>>  = Vec::new();

    // tmp.push(Box::new(stdin));
    // tmp.push(Box::new(socket));

    

    // loop {
    //     for socket in tmp.iter() {
    //         let mut data = String::new();
    //         match socket.read_date(&mut data) {
    //             Ok(_) => {}
    //             Err(_) => {}
    //         }
    //     }
    // }

    let header_bytes: [u8; 16] = [
        0x00, 0x50, // Source port (80)
        0x01, 0xBB, // Destination port (443)
        0x00, 0x00, 0x00, 0x01, // Sequence number
        0x00, 0x00, 0x00, 0x00, // Acknowledgment number
        0x50, // Header length (5) and Reserved (0), here for simplicity
        0b0011_1111, // Flags (URG, ACK, PSH, RST, SYN, FIN)
        0x00, 0x00, // Window size (just an example)
    ];

    let tcp_header = TcpHeader::new(&header_bytes);
    dbg!(&tcp_header);
    let copy_header = TcpHeader::new(tcp_header.as_bytes().as_slice());
    dbg!(&copy_header);


    // Ok(())
    let data_len = handle.read_line(&mut buffer).map_err(|e| format!("{e} -> failed to read"))?;
    // println!("Read {} bytes", data_len);
    // println!("{} from rust", buffer);

    let texts: Vec<String> = buffer.as_bytes().chunks(1500).map(| ch| String::from_utf8_lossy(ch).to_string()).collect();

    let mut acks: Vec<usize> = vec![0; texts.len()];

    for text in texts.iter() {
        socket.send_to(text.as_bytes() , format!("{}:{}", cli.recv_host.clone(), cli.recv_port.clone())).unwrap();
    }

    // socket.send_to("jazzcort".as_bytes(), format!("{}:{}", cli.recv_host.clone(), cli.recv_port.clone())).unwrap();

    let mut buf: [u8; 1500] = [0; 1500];
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



    
   
    // let mut handles: Vec<JoinHandle<()>> = vec!();
    // let host = cli.recv_host.clone();
    // let port = cli.recv_port.clone();
    
    
    // let mut i = 0;

    // for mut t in texts.into_iter() {
    //     let socket_copy = socket.try_clone().unwrap();
    //     let a = host.clone();
    //     let b = port.clone();
    //     // t.push_str(format!("***{}***", i).as_str());
    //     // dbg!(&t);
    //     i += 1;

    //     // let (host, port) = (&cli.recv_host.clone(), cli.recv_port.clone());
    //     let handle = spawn(move || {
    //         socket_copy.send_to(t.as_bytes() , format!("{}:{}", a, b)).unwrap();
    //     });

    //     handles.push(handle);
    // }

    // for handle in handles {
    //     handle.join().unwrap();
    // }


    // eprintln!("{}", first);

    // match socket.send_to(first.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)) {
    //     Ok(_) => {eprintln!("Success")}
    //     Err(_) => {eprintln!("Fail")}
    // }
    // scoket.send_to(second.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)).unwrap();
    Ok(())
}
