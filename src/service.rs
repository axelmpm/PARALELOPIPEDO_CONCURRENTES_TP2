use crate::service_kind::{ServiceKind, kind_address};
use crate::message::deserialize;
use crate::processor::Processor;

use std::net::{TcpListener, TcpStream};
use std::io::{BufRead, BufReader};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use crate::logger::Logger;
use std::io::Write;
use std::time::Duration;

pub struct Service {
    kind: ServiceKind,
    listener: TcpListener,
    processor: Arc<Processor>,
    closed: AtomicBool,
    logger: Arc<Mutex<Logger>>
}

impl Service {

    pub fn new(kind: ServiceKind) -> Service{
        let address = format!("localhost:{}", kind_address(kind));
        let logger = Arc::new(Mutex::new(Logger::new(kind.to_string() + ".txt")));
        return Service{
            kind,
            listener: TcpListener::bind(address).unwrap(), 
            processor: Arc::new(Processor::new(logger.clone())),
            closed: AtomicBool::new(false),
            logger,
        }
    }

    pub fn close(& self) {
       self.closed.store(true, Ordering::Relaxed);
    }

    pub fn run(&self, ctrlc_event: Arc<Mutex<Receiver<()>>>) {

        println!("service started");

        self.listener.set_nonblocking(true).expect("Cannot set non-blocking");
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => self.handle_connection(stream, ctrlc_event.clone()),
                Err(ref e) if e.kind() == WouldBlock => {
                    if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                        break; 
                    }
                    continue;
                },
                Err(e) => panic!("{}", e),
            }
        }

    }

    fn handle_connection(&self, mut stream: TcpStream, ctrlc_event: Arc<Mutex<Receiver<()>>>){
        println!("new connection");

        stream.set_nonblocking(true).expect("Cannot set non-blocking");
        let mut reader = BufReader::new(stream.try_clone().expect("could not clone stream"));
        for line in reader.lines(){

            match line {
                Ok(line) => {
                    let message = deserialize(line);
                    let response = self.processor.process(message);
                    stream.write_all(response.serialize().as_bytes()).unwrap();
                }
                Err(ref e) if e.kind() == WouldBlock => {
                    if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                        break; 
                    }
                    continue;
                },

                Err(e) => panic!("{}", e),
            }
        }
    }
}

