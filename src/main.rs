

fn main() {
    lib::run();
}

/*
// helloword_server.rs
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    for stream in listener.incoming() {
        println!("Cliente conectado");
        let mut reader = BufReader::new(stream.unwrap());
        thread::spawn(move || {
            loop {
                let mut buffer = String::new();
                reader.read_line(&mut buffer);
                if buffer.len() > 0 {
                    println!("Hello {}", buffer);
                } else {
                    println!("Goodbye!");
                    break;
                }
            }
        });
    }
}



// helloword.rs
use std::io::Write;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use std::env;

fn main() {

    let mut stream = TcpStream::connect("127.0.0.1:12345").unwrap();
    println!("Conectado");

    loop {
        println!("Enviando");
        stream.write_all(env::args().skip(1).next().unwrap().as_bytes()).unwrap();
        stream.write_all("\n".as_bytes()).unwrap();
        sleep(Duration::from_secs(1))
    }

}*/