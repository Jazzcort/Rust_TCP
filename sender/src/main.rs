use std::io::{self, BufRead};
use clap::Parser;
use std::net::UdpSocket;

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    recv_host: String,
    recv_port: String
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let scoket = UdpSocket::bind("127.0.0.1:0").unwrap();


    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let data_len = handle.read_line(&mut buffer).map_err(|e| format!("{e} -> failed to read"))?;
    // println!("Read {} bytes", data_len);
    // println!("{} from rust", buffer);

    let (first, second) = buffer.split_at(1480);
    eprintln!("{}", first);

    match scoket.send_to(first.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)) {
        Ok(_) => {eprintln!("Success")}
        Err(_) => {eprintln!("Fail")}
    }
    // scoket.send_to(second.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)).unwrap();
    Ok(())
}
