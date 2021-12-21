use crate::message::Message;
use crate::message_kind::MessageKind;
use std::collections::HashMap;
extern crate rand;
use crate::logger::Logger;
use crate::processor::rand::Rng;
use crate::service_kind::ServiceKind;
use core::time::Duration;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Processor {
    logger: Arc<Mutex<Logger>>,
    results: HashMap<i32, Message>
}

impl Processor {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Processor {
        Processor {
            logger,
            results: HashMap::new()
        }
    }

    pub fn process(&mut self, message: Message, service: ServiceKind) -> Message {
        match message.kind {
            // Esto sería equivalente a un COMMIT
            MessageKind::Confirmation => {
                self.logger
                    .lock()
                    .unwrap_or_else(|_| panic!("PROCESSOR: couldnt adquiere lock"))
                    .write_line(format!(
                        "{} COMMIT {}",
                        service.to_string(),
                        message.body.id.to_string()
                    ));
                Message::new(MessageKind::Ack, message.body)
                // accept from pending storage
            }

            // Esto seria equivalente a un ROLLBACK
            MessageKind::Rejection => {
                self.logger
                    .lock()
                    .unwrap_or_else(|_| panic!("PROCESSOR: couldnt adquiere lock"))
                    .write_line(format!(
                        "{} ABORT {}",
                        service.to_string(),
                        message.body.id.to_string()
                    ));
                Message::new(MessageKind::Ack, message.body)
                // reject from pending storage
            }

            // Esto seria equivalente a un PREPARE
            MessageKind::Transaction => {
                thread::sleep(Duration::from_millis(
                    rand::thread_rng().gen_range(500..2000),
                ));
                

                match self.results.get(&message.body.id) {
                    Some(result) => result.clone(),
                    None => {
                        let id = message.body.id;
                        let result = self.process_new_transaction(message, service);
                        self.results.insert(id, result.clone());
                        result
                    }
                }      
            }

            MessageKind::Retry => {
                let result = self.process_new_transaction(message, service);
                return result;
            }

            _ => Message::new(MessageKind::Rejection, message.body),
        }


    }
    
    fn process_new_transaction(&self, message: Message, service: ServiceKind) -> Message{
        let luck = rand::thread_rng().gen_range(0..10);
        if luck > 2 {
            //accepted
            //println!("PROCESSOR: message.body = {}", message.body);
            self.logger
                .lock()
                .unwrap_or_else(|_| panic!("PROCESSOR: couldnt adquiere lock"))
                .write_line(format!(
                    "{} ACCEPTED {}",
                    service.to_string(),
                    message.body.id.to_string()
                ));
            Message::new(MessageKind::Confirmation, message.body)
        } else {
            //rejected
            //println!("PROCESSOR: message.body = {}", message.body);
            self.logger
                .lock()
                .unwrap_or_else(|_| panic!("PROCESSOR: couldnt adquiere lock"))
                .write_line(format!(
                    "{} REJECTED {}",
                    service.to_string(),
                    message.body.id.to_string()
                ));
            Message::new(MessageKind::Rejection, message.body)
        }
    }
}
