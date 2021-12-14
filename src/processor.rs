use crate::message::Message;
use crate::message_kind::MessageKind;
use crate::pending_storage::PendingStorage;

extern crate rand;
use crate::logger::Logger;
use crate::processor::rand::Rng;
use core::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::service_kind::ServiceKind;

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

    pub fn process(&self, message: Message, service: ServiceKind) -> Message {
        match message.kind {
            // Esto sería equivalente a un COMMIT
            MessageKind::Confirmation => {
                self.logger
                    .lock()
                    .unwrap()
                    .write_line(format!("{} COMMIT {}", service.to_string(), message.body.id.to_string()));
                Message::new(MessageKind::Ack, message.body)
                // accept from pending storage
            }

            // Esto seria equivalente a un ROLLBACK
            MessageKind::Rejection => {
                self.logger
                    .lock()
                    .unwrap()
                    .write_line(format!("{} ABORT {}", service.to_string(), message.body.id.to_string()));
                Message::new(MessageKind::Ack, message.body)
                // reject from pending storage
            }

            // Esto seria equivalente a un PREPARE
            MessageKind::Transaction => {
                thread::sleep(Duration::from_millis(
                    rand::thread_rng().gen_range(500..2000),
                ));
                let luck = rand::thread_rng().gen_range(0..10);
                self.storage
                    .lock()
                    .unwrap()
                    .store(message.body.id.to_string());
                if luck > 2 {
                    //accepted
                    //println!("PROCESSOR: message.body = {}", message.body);
                    self.logger
                        .lock()
                        .unwrap()
                        .write_line(format!("{} ACCEPTED {}", service.to_string(), message.body.id.to_string()));
                    Message::new(MessageKind::Confirmation, message.body)
                } else {
                    //rejected
                    //println!("PROCESSOR: message.body = {}", message.body);
                    self.logger
                        .lock()
                        .unwrap()
                        .write_line(format!("{} REJECTED {}", service.to_string(), message.body.id.to_string()));
                    Message::new(MessageKind::Rejection, message.body)
                }
            }

            _ => Message::new(MessageKind::Rejection, message.body),
        }
    }
}
