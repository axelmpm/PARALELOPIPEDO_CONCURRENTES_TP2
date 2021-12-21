use crate::message::deserialize;
use crate::processor::Processor;
use crate::service_kind::{kind_address, ServiceKind};

use crate::logger::Logger;
use std::io::ErrorKind::WouldBlock;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct Service {
    kind: ServiceKind,
    processor: Processor,
    address: String,
    _logger: Arc<Mutex<Logger>>,
}

impl Service {
    pub fn new(kind: ServiceKind) -> Service {
        let address = format!("localhost:{}", kind_address(kind));
        let logger = Arc::new(Mutex::new(Logger::new(kind.to_string() + ".txt")));
        Service {
            kind,
            address: address,
            processor: Processor::new(logger.clone()),
            _logger: logger,
        }
    }

    pub fn run(&mut self, ctrlc_event: Arc<Mutex<Receiver<()>>>) {
        println!("service started");

        let listener = TcpListener::bind(self.address.clone())
        .unwrap_or_else(|_| panic!("SERVICE: {} couldn bind tcp conection", self.kind));

        listener
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => self.handle_connection(stream, ctrlc_event.clone()),
                Err(ref e) if e.kind() == WouldBlock => {
                    if ctrlc_event
                        .lock()
                        .unwrap_or_else(|_| panic!("SERVICE: {} couldnt adquiere lock", self.kind))
                        .try_recv()
                        .is_ok()
                    {
                        //received ctrlc
                        break;
                    }
                    continue;
                }
                Err(_e) => continue,
            }
        }
    }

    fn handle_connection(&mut self, mut stream: TcpStream, ctrlc_event: Arc<Mutex<Receiver<()>>>) {
        //println!("new connection");

        stream
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");
        let reader = BufReader::new(stream.try_clone().expect("could not clone stream"));
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let message = deserialize(line);
                    let response = self.processor.process(message, self.kind);
                    if let Err(e) = stream.write_all(response.serialize().as_bytes()) {
                        println!("{}", e);
                        break;
                    }
                }
                Err(ref e) if e.kind() == WouldBlock => {
                    if ctrlc_event
                        .lock()
                        .unwrap_or_else(|_| panic!("SERVICE: couldnt adquiere lock"))
                        .try_recv()
                        .is_ok()
                    {
                        //received ctrlc
                        break;
                    }
                    continue;
                }

                Err(e) => {
                    println!("{}", e);
                    break;
                }
            }
        }
    }
}
