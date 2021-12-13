use crate::service_kind::{ServiceKind, kind_address};
use crate::message_body::MessageBody;
use crate::message::{Message, deserialize};
use crate::message_kind::MessageKind;
use crate::logger::Logger;
use crate::transaction_log_parser::TransactionLogParser;
use crate::transaction_parser::TransactionParser;
use crate::transaction_phase::TransactionPhase;
use std::fs::read_dir;
use std::net::{TcpStream};
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::mpsc::Receiver;
use std::thread;
use std::collections::HashMap;
use crate::transaction::Transaction;
use crate::leader_election::LeaderElection;


pub struct Alglobo {
    host: String,
    port: i32,
    service_streams: HashMap<ServiceKind, TcpStream>,
    failed_transactions: HashMap<i32, Arc<Transaction>>,
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
        
        return Alglobo{host, port, service_streams, failed_transactions: HashMap::new()};
    }

    pub fn retry(&self, id: i32) {
        if self.failed_transactions.contains_key(&id) {
            let transaction = self.failed_transactions.get(&id).unwrap_or_else(|| panic!("ALGLOBO: INTERNAL ERROR"));
        }

        // do retry TODO
    }

    pub fn process(&mut self, ctrlc_event: Arc<Mutex<Receiver<()>>>) -> bool{

        let ctrlc_pressed = Arc::new(Mutex::new(false));
        let ctrlc_pressed_copy = ctrlc_pressed.clone();

        let mut transaction_parser = TransactionParser::new("transactions.txt".to_owned());
        let mut transaction_log = Logger::new("transaction_log.txt".to_owned());

        let leader_election = LeaderElection::new(self.port as u32); //todo get id from somewhere
        let leader_clone = leader_election.clone();
        let leader_thread = thread::spawn(move || leader_clone.work());
        let mut last_processed_transaction: Option<(i32, TransactionPhase)>;
        loop {
            if !leader_election.am_i_leader() {

                leader_election.wait_until_leader_changes();

                if leader_election.am_i_leader() {
                    // Continue with transcation left from last leader
                    last_processed_transaction = TransactionLogParser::new().get_last_transaction();

                    if let Some((id, phase)) = last_processed_transaction {
                        if let Some(transaction) = transaction_parser.seek_transaction(id) {
                            let transaction = Arc::new(transaction);
                            // If transaction was left at INIT stage
                            if phase == TransactionPhase::Init {
                                let responses = self.process_operations(transaction.clone(), MessageKind::Transaction);
                                if responses.contains(&MessageKind::Rejection) {
                                    transaction_log.write_line(format!("ABORT {}", transaction.id));
                                    self.failed_transactions.entry(transaction.id).or_insert_with(|| transaction.clone());
                                } else {
                                    transaction_log.write_line(format!("COMMIT {}", transaction.id));
                                    self.process_operations(transaction, MessageKind::Confirmation);
                                }
                            } else if phase == TransactionPhase::Abort { // If transaction was left at ABORT stage
                                self.process_operations(transaction, MessageKind::Rejection);
                            } else if phase == TransactionPhase::Commit { // If transaction was left at COMMIT stage
                                self.process_operations(transaction, MessageKind::Confirmation);
                            }
                            if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                                *ctrlc_pressed_copy.lock().unwrap() = true;
                                leader_election.close();
                                break;
                            }
                        }
                    }
                }
            } else if let Some(transaction) = transaction_parser.read_transaction() {
                let transaction = Arc::new(transaction);
                transaction_log.write_line(format!("INIT {}", transaction.id));
                let responses = self.process_operations(transaction.clone(), MessageKind::Transaction);

                if responses.contains(&MessageKind::Rejection) {
                    transaction_log.write_line(format!("ABORT {}", transaction.id));
                    self.failed_transactions.entry(transaction.id).or_insert_with(|| transaction.clone());
                    self.process_operations(transaction, MessageKind::Rejection);
                } else {
                    transaction_log.write_line(format!("COMMIT {}", transaction.id));
                    self.process_operations(transaction, MessageKind::Confirmation);
                }

                if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                    *ctrlc_pressed_copy.lock().unwrap() = true;
                    leader_election.close();
                    break;
                }
            }
        }
        return *ctrlc_pressed.lock().unwrap();
    }

    fn process_operations(&self, transaction: Arc<Transaction>, kind: MessageKind) -> Vec<MessageKind>{

        let mut loglist = vec![];
        for operation in &transaction.operations {
            let body = MessageBody::new(transaction.id, operation.service, operation.amount as i32, 0);
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
        return loglist;
    }
}

