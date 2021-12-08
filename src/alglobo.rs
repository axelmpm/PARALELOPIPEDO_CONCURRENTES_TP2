use crate::service_kind::{ServiceKind, kind_address};
use crate::message_body::{body_parser, MessageBody};
use crate::message::{Message, deserialize};
use crate::message_kind::MessageKind;
use crate::logger::Logger;
use crate::transaction_parser::TransactionParser;
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
use crate::transaction::Transaction;
use crate::leader_election::LeaderElection;


pub struct Alglobo {
    host: String,
    port: i32,
    service_streams: HashMap<ServiceKind, TcpStream>,
    failed_transactions: Arc<Mutex<HashMap<i32, VecDeque<MessageBody>>>>,
}

impl Alglobo {

    pub fn new(host: String, port: i32) -> Self {
        let hotel_address = format!("localhost:{}", kind_address(ServiceKind::Hotel));
        let bank_address: String = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let airline_address: String = format!("localhost:{}", kind_address(ServiceKind::Airline));

        let mut service_streams = HashMap::new();
        service_streams.insert(ServiceKind::Hotel, TcpStream::connect(hotel_address).expect("No fue posible conectarse a servicio de Hotel"));
        service_streams.insert(ServiceKind::Bank, TcpStream::connect(bank_address).expect("No fue posible conectarse a servicio de Banco"));
        service_streams.insert(ServiceKind::Airline, TcpStream::connect(airline_address).expect("No fue posible conectarse a servicio de Aerolinea"));
        
        return Alglobo{host, port, service_streams, failed_transactions: Arc::new(Mutex::new(HashMap::new()))};
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

        // let mut hotel_reader = BufReader::new(self.hotel_stream.try_clone().expect("could not clone stream"));
        /*
        let airline_address = format!("localhost:{}", kind_address(ServiceKind::Airline));
        let mut airline_stream = TcpStream::connect(airline_address).unwrap();

        let bank_address = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let mut bank_stream = TcpStream::connect(bank_address).unwrap();*/

        //parsear archivo transacciones


        let mut transaction_parser = TransactionParser::new("transactions.txt".to_owned());

        let mut rejected = Logger::new("rejections.txt".to_owned());
        let mut accepted = Logger::new("accepted.txt".to_owned());
        let mut transaction_log = Logger::new("transaction_log.txt".to_owned());
        
        // let incoming_message_listener_thread = thread::spawn(move || {
            
        //     loop {
        //         let mut buffer = String::new();
        //         hotel_reader.read_line(&mut buffer); //is blocking unless connection down!!!
        //         if buffer.len() > 0 {
        //             let incoming_message = deserialize(buffer);
    
        //             match incoming_message.kind.clone() {
        //                 MessageKind::Confirmation => {
        //                     accepted.log(incoming_message.body);
        //                 },
            
        //                 MessageKind::Rejection => {
        //                     failed_transactions.lock().unwrap().entry(incoming_message.body.id.clone()).or_insert_with(|| VecDeque::new()).push_back(incoming_message.body.clone());
        //                     rejected.log(incoming_message.body);
        //                 },
            
        //                 _ => {},
        //             }
        //         } else {
        //             break; // conection down. Stops listening messages
        //         } 

        //         if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
        //             *ctrlc_pressed_copy.lock().unwrap() = true;
        //             break;
        //         }
        //     }
        // });

        let leader_election = LeaderElection::new(self.port as u32); //todo get id from somewhere
        let leader_clone = leader_election.clone();
        let leader_thread = thread::spawn(move || {
            loop {
                leader_clone.work();
                //todo check and keep cycling while not ctrlc
            }
        });

        loop {
            if !leader_election.am_i_leader() {

                leader_election.wait_until_leader_changes();

            } else if let Some(transaction) = transaction_parser.read_transaction() {

                let transaction = Arc::new(transaction);
                let mut responses = vec![];
                transaction_log.write_line(format!("INIT {}", transaction.id));
                self.process_operations(transaction.clone(), MessageKind::Transaction, responses);

                if responses.contains(&MessageKind::Rejection) {
                    transaction_log.write_line(format!("ABORT {}", transaction.id));
                    let mut acks = vec![];
                    self.process_operations(transaction, MessageKind::Rejection, acks);
                } else {
                    transaction_log.write_line(format!("COMMIT {}", transaction.id));
                    let mut acks = vec![];
                    self.process_operations(transaction, MessageKind::Confirmation, acks);
                }

                if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                    *ctrlc_pressed_copy.lock().unwrap() = true;
                    break;
                }

            }
        }

        // incoming_message_listener_thread.join();

        return *ctrlc_pressed.lock().unwrap();
    }

    fn process_operations(&self, transaction: Arc<Transaction>, kind: MessageKind, &mut loglist: Vec<MessageKind>){
        for operation in &transaction.operations {
            let body = MessageBody::new(transaction.id.parse::<i32>().unwrap(), operation.service, operation.amount as i32, 0);
            let message = Message::new(kind, body);
            let mut service_stream = self.service_streams.get_key_value(&operation.service).unwrap().1;
            service_stream.write_all(message.serialize().as_bytes()).unwrap();

            let mut reader = BufReader::new(service_stream.try_clone().expect("could not clone stream"));
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();

            if buffer.len() > 0 {
                let incoming_message = deserialize(buffer);
                loglist.push(incoming_message.kind);
            }
        }
    }
}

