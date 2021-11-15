use crate::service_kind::{ServiceKind, kind_address};

use std::net::TcpListener;
use std::io::{BufRead, BufReader};
use std::thread;
use std::io::Write;


pub struct Service {
    kind: ServiceKind,
}

impl Service {

    pub fn new(kind: ServiceKind) -> Service{
        return Service{kind};
    }

    pub fn run(&self) {

        println!("Conectado");

        let address = format!("localhost:{}", kind_address(self.kind));
        let listener = TcpListener::bind(address).unwrap();

        for stream in listener.incoming() {
            println!("new connection");
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().expect("could not clone stream"));

            thread::spawn(move || {
                loop {
                    let mut buffer = String::new();
                    reader.read_line(&mut buffer);
                    if buffer.len() > 0 {
                        let mut stream = stream.try_clone().expect("could not clone stream");
                        thread::spawn(move ||{
                            
                            //thread sleep random
                            
                            println!("Hello {}", buffer);
                            stream.write_all(format!("SUCCESS {}", buffer).as_bytes());
                        });
                    } else {
                        println!("Goodbye!");
                        break;
                    }
                }
            });
        }
    }
}
