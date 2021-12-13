use crate::service_kind::{ServiceKind, kind_address};
use crate::message_body::MessageBody;
use crate::message::{Message, deserialize};
use crate::message_kind::MessageKind;
use crate::logger::Logger;
use crate::transaction_parser::TransactionParser;
use std::net::{TcpStream};
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::collections::HashMap;
use crate::transaction::Transaction;
use crate::leader_election::{LeaderElection, N_NODES};


pub struct Alglobo {
    host: String,
    port: i32,
    service_streams: Arc<Mutex<HashMap<ServiceKind, TcpStream>>>,
    failed_transactions: Arc<Mutex<HashMap<i32, Arc<Transaction>>>>,
    transaction_log: Arc<Mutex<Logger>>,
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

        let transaction_log = Logger::new("transaction_log.txt".to_owned());
        
        return Alglobo{
            host,
            port,
            service_streams: Arc::new(Mutex::new(service_streams)),
            failed_transactions: Arc::new(Mutex::new(HashMap::new())),
            transaction_log: Arc::new(Mutex::new(transaction_log))
        };
    }

    pub fn clone(&self)-> Alglobo {
        return Alglobo{
            host: self.host.clone(),
            port: self.port,
            service_streams: self.service_streams.clone(),
            failed_transactions:self.failed_transactions.clone(),
            transaction_log: self.transaction_log.clone()
        }
    }

    pub fn retry(&mut self, id: i32) -> bool{
        if self.failed_transactions.contains_key(&id) {
            let transaction = self.failed_transactions.get(&id).unwrap_or_else(|| panic!("ALGLOBO: INTERNAL ERROR"));
            if self.process_transaction(transaction.clone()){
                self.failed_transactions.remove_entry(&id).unwrap();
            }
        } else {
            return false;
        }
        return true;
    }

    pub fn process(&mut self, ctrlc_event: Arc<Mutex<Receiver<()>>>) -> bool{

        let ctrlc_pressed = Arc::new(Mutex::new(false));
        let ctrlc_pressed_copy = ctrlc_pressed.clone();

        let mut transaction_parser = TransactionParser::new("transactions.txt".to_owned());

        let mut handles = vec!();
        for id in 0..N_NODES {

            let leader_election = LeaderElection::new(self.host.clone(), self.port as u32, id); //todo get id from somewhere
            let leader_clone = leader_election.clone();
            let leader_thread = thread::spawn(move || leader_clone.work());
            let mut alglobo = self.clone();

            handles.push(thread::spawn(move || {
                loop {
                    if ctrlc_event.lock().unwrap().try_recv().is_ok() { //received ctrlc
                        *ctrlc_pressed_copy.lock().unwrap() = true;
                        leader_election.close();
                        break;
                    } else if !leader_election.am_i_leader() {
                        leader_election.wait_until_leader_changes();
                    } else if let Some(transaction) = transaction_parser.read_transaction() {
                        alglobo.process_transaction(Arc::new(transaction));
                    } else {
                        break; // no more transacitions
                    }
                }
            }));
        }
        handles.into_iter().for_each(|h| { h.join(); });

        leader_election.close();
        return *ctrlc_pressed.lock().unwrap();
    }

    fn process_transaction(&mut self, transaction: Arc<Transaction>) -> bool{

        self.transaction_log.write_line(format!("INIT {}", transaction.id));
        let responses = self.process_operations(transaction.clone(), MessageKind::Transaction);

        if responses.contains(&MessageKind::Rejection) {
            self.transaction_log.write_line(format!("ABORT {}", transaction.id));
            self.failed_transactions.entry(transaction.id).or_insert_with(|| transaction.clone());
            self.process_operations(transaction, MessageKind::Rejection);
            return false;
        } else {
            self.transaction_log.write_line(format!("COMMIT {}", transaction.id));
            self.process_operations(transaction, MessageKind::Confirmation);
            return true;
        }
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

    pub fn show_failed_transactions(&self){

        if self.failed_transactions.len() == 0 {
            println!("no failed transactions");
        } else {
            
            for (key, value) in &self.failed_transactions{
                println!("[TRANSCACTION {}] {}", key, value);
            }
        }
    }
}

