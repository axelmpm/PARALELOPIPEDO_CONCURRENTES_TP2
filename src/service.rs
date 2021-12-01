use crate::service_kind::{ServiceKind, kind_address};
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

        println!("Conectado");
        
        self.listener.set_nonblocking(true).expect("Cannot set non-blocking");
        let mut streams_threads = vec![];
        
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) =>  streams_threads.push(self.handle_connection(stream, ctrlc_event.clone())),

                Err(ref e) if e.kind() == WouldBlock => {
                    if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                        break; 
                    }
                    continue;
                },

                Err(e) => panic!("encountered IO error: {}", e),
            }
        }

        for streams_thread in streams_threads {
            println!("joined streams_thread");
            streams_thread.join();
        }
    }

    fn handle_connection(&self, stream: TcpStream, ctrlc_event: Arc<Mutex<Receiver<()>>>) -> JoinHandle<()>{
        println!("new connection");
        let mut reader = BufReader::new(stream.try_clone().expect("could not clone stream"));

        let processor = self.processor.clone();
        return thread::spawn(move || {

            let mut buffer_line_threads = vec![];
            loop {
                let mut buffer = String::new();
                reader.read_line(&mut buffer);
                if buffer.len() > 0 {
                    let stream = stream.try_clone().expect("could not clone stream");
                    let processor = processor.clone();
    
                    buffer_line_threads.push(thread::spawn(move ||{
                      processor.process(buffer, stream);
                    }));
                }

                if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                    break; 
                }
            }

            for buffer_line in buffer_line_threads {
                println!("joined buffer_line");
                buffer_line.join();
            }
        }
        );
    }
}
