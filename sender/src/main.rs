use std::io::{self, BufRead};
use clap::Parser;
use std::net::UdpSocket;
use std::thread::{JoinHandle, spawn};

#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
struct Cli {
    recv_host: String,
    recv_port: String
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    


    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let data_len = handle.read_line(&mut buffer).map_err(|e| format!("{e} -> failed to read"))?;
    println!("Read {} bytes", data_len);
    // println!("{} from rust", buffer);

    let texts: Vec<String> = buffer.as_bytes().chunks(1500).map(| ch| String::from_utf8_lossy(ch).to_string()).collect();
   
    let mut handles: Vec<JoinHandle<()>> = vec!();
    let host = cli.recv_host.clone();
    let port = cli.recv_port.clone();
    
    
    let mut i = 0;

    for mut t in texts.into_iter() {
        let socket_copy = socket.try_clone().unwrap();
        let a = host.clone();
        let b = port.clone();
        // t.push_str(format!("***{}***", i).as_str());
        // dbg!(&t);
        i += 1;

        // let (host, port) = (&cli.recv_host.clone(), cli.recv_port.clone());
        let handle = spawn(move || {
            socket_copy.send_to(t.as_bytes() , format!("{}:{}", a, b)).unwrap();
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }


    // eprintln!("{}", first);

    // match socket.send_to(first.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)) {
    //     Ok(_) => {eprintln!("Success")}
    //     Err(_) => {eprintln!("Fail")}
    // }
    // scoket.send_to(second.as_bytes(), format!("{}:{}", cli.recv_host, cli.recv_port)).unwrap();
    Ok(())
}
