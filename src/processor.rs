
use crate::message_kind::MessageKind;
use crate::message::{Message, deserialize};
use crate::pending_storage::PendingStorage;

extern crate rand;
use crate::processor::rand::Rng;
use std::thread;
use core::time::Duration;
use std::net::TcpStream;
use std::io::Write;


pub struct Processor{
    storage: PendingStorage
}

impl Processor {
    pub fn new() -> Processor {
        Processor{storage: PendingStorage::new()}
    }

    pub fn process(&self, buffer: String, mut stream: TcpStream){

        let message = deserialize(buffer);

        match message.kind.clone() {
            MessageKind::Confirmation => {
                // accept from pending storage
            },

            MessageKind::Rejection => {
                // reject from pending storage
            },

            MessageKind::Transaction => {
                thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(500..2000)));
                let luck = rand::thread_rng().gen_range(0..10);
                if luck > 5 {
                    //accepted
                    println!("accepted!");
                    println!("PROCESSOR: message.body = {}", message.body);
                    stream.write_all(Message::new(MessageKind::Confirmation, message.body).serialize().as_bytes());
                } else {
                    //rejected
                    println!("rejected!");
                    println!("PROCESSOR: message.body = {}", message.body);
                    stream.write_all(Message::new(MessageKind::Rejection, message.body).serialize().as_bytes());
                }
            },
        }
    }
}