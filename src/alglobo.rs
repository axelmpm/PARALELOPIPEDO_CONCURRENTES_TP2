use crate::service_kind::{ServiceKind, kind_address};
use crate::message_body::{body_parser, MessageBody};
use crate::message::{Message, deserialize};
use crate::message_kind::MessageKind;
use crate::logger::Logger;
use std::net::{TcpStream};
use std::io::Write;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::collections::VecDeque;
use std::collections::HashMap;


pub struct Alglobo {
    host: String,
    port: i32,
    hotel_stream: TcpStream,
    failed_transactions: Arc<Mutex<HashMap<i32, VecDeque<MessageBody>>>>,
}

impl Alglobo {

    pub fn new(host: String, port: i32) -> Self {
        let hotel_address = format!("localhost:{}", kind_address(ServiceKind::Hotel));
        let hotel_stream = TcpStream::connect(hotel_address).unwrap();
        return Alglobo{host, port, hotel_stream, failed_transactions: Arc::new(Mutex::new(HashMap::new()))};
    }

    pub fn retry(&self, id: i32) {
        if self.failed_transactions.lock().unwrap().contains_key(&id) {
            let transactions = self.failed_transactions.lock().unwrap().get(&id).unwrap_or_else(|| panic!("ALGLOBO: INTERNAL ERROR"));
        }
    }

    pub fn process(&mut self, ctrlc_event: Arc<Mutex<Receiver<()>>>) -> bool{

        let ctrlc_pressed = Arc::new(Mutex::new(false));
        let ctrlc_pressed_copy = ctrlc_pressed.clone();

        let failed_transactions = self.failed_transactions.clone();

        let mut hotel_reader = BufReader::new(self.hotel_stream.try_clone().expect("could not clone stream"));
        /*
        let airline_address = format!("localhost:{}", kind_address(ServiceKind::Airline));
        let mut airline_stream = TcpStream::connect(airline_address).unwrap();

        let bank_address = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let mut bank_stream = TcpStream::connect(bank_address).unwrap();*/

        //parsear archivo transacciones

        let file = File::open("transactions.txt").expect("Problem opening file");
        let mut rejected = Logger::new("rejections.txt".to_owned());
        let mut accepted = Logger::new("accepted.txt".to_owned());
        
        let reader = BufReader::new(file);

        let incoming_message_listener_thread = thread::spawn(move || {
            
            loop {
                let mut buffer = String::new();
                hotel_reader.read_line(&mut buffer); //is blocking unless connection down!!!
                if buffer.len() > 0 {
                    let incoming_message = deserialize(buffer);
    
                    match incoming_message.kind.clone() {
                        MessageKind::Confirmation => {
                            accepted.log(incoming_message.body);
                        },
            
                        MessageKind::Rejection => {
                            failed_transactions.lock().unwrap().entry(incoming_message.body.id.clone()).or_insert_with(|| VecDeque::new()).push_back(incoming_message.body.clone());
                            rejected.log(incoming_message.body);
                        },
            
                        _ => {},
                    }
                } else {
                    break; // conection down. Stops listening messages
                } 

                if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                    *ctrlc_pressed_copy.lock().unwrap() = true;
                    break;
                }
            }
        });

        for line in reader.lines().flatten() {
        
            let body = body_parser(line);
            println!("body = {}", body);
            self.hotel_stream.write_all(Message::new(MessageKind::Transaction, body).serialize().as_bytes()); //TODO handelear error y reconectarse
        }

        incoming_message_listener_thread.join();

        return *ctrlc_pressed.lock().unwrap();
    }
}

