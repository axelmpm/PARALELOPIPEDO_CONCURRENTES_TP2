
use crate::message_kind::MessageKind;
use crate::message::{Message, deserialize};
use crate::pending_storage::PendingStorage;

extern crate rand;
use crate::processor::rand::Rng;
use crate::logger::{Logger};
use std::thread;
use std::sync::{Arc,Mutex};
use core::time::Duration;
use std::net::TcpStream;
use std::io::Write;


pub struct Processor {
    storage: Mutex<PendingStorage>,
    logger: Arc<Mutex<Logger>>,
}

impl Processor {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Processor {
        Processor {
            storage: Mutex::new(PendingStorage::new()),
            logger,
        }
    }

    pub fn process(&self, buffer: String, mut stream: TcpStream){

        let message = deserialize(buffer);
        match message.kind.clone() {
            // Esto sería equivalente a un COMMIT
            MessageKind::Confirmation => {
                self.logger.lock().unwrap().write_line(format!("COMMIT {}", message.body.id.to_string()));
                stream.write_all(Message::new(MessageKind::Ack, message.body).serialize().as_bytes()).unwrap();
                // accept from pending storage
            },

            // Esto seria equivalente a un ROLLBACK
            MessageKind::Rejection => {
                self.logger.lock().unwrap().write_line(format!("ABORT {}", message.body.id.to_string()));
                stream.write_all(Message::new(MessageKind::Ack, message.body).serialize().as_bytes()).unwrap();
                // reject from pending storage
            },

            // Esto seria equivalente a un PREPARE
            MessageKind::Transaction => {
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(500..2000)));
                let luck = rand::thread_rng().gen_range(0..10);
                self.storage.lock().unwrap().store(message.body.id.to_string());
                if luck > 5 {
                    //accepted
                    println!("accepted!");
                    println!("PROCESSOR: message.body = {}", message.body);
                    self.logger.lock().unwrap().write_line(format!("ACCEPTED {}", message.body.id.to_string()));
                    stream.write_all(Message::new(MessageKind::Confirmation, message.body).serialize().as_bytes()).unwrap();
                } else {
                    //rejected
                    println!("rejected!");
                    println!("PROCESSOR: message.body = {}", message.body);
                    self.logger.lock().unwrap().write_line(format!("REJECTED {}", message.body.id.to_string()));
                    stream.write_all(Message::new(MessageKind::Rejection, message.body).serialize().as_bytes()).unwrap();
                }
            },

            _ => {}
        }
    }
}