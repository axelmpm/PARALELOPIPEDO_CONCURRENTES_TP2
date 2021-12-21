use crate::leader_election::LeaderElection;
use crate::logger::Logger;
use crate::message::{deserialize, Message};
use crate::message_body::MessageBody;
use crate::message_kind::MessageKind;
use crate::service_kind::{kind_address, ServiceKind};
use crate::transaction::Transaction;
use crate::transaction_log_parser::TransactionLogParser;
use crate::transaction_parser::TransactionParser;
use crate::transaction_phase::TransactionPhase;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::service_stream::ServiceStream;

pub struct Alglobo {
    host: String,
    port: i32,
    id: u32,
    failed_transactions: HashMap<i32, Arc<Transaction>>,
    failed_transaction_log: Logger,
    transaction_log: Logger,
}

impl Alglobo {
    pub fn new(host: String, port: i32, id: u32) -> Self {
        let transaction_log = Logger::new("transaction_log.txt".to_owned());
        let failed_transaction_log = Logger::new("failed_transactions_log.txt".to_owned());

        Alglobo {
            host,
            port,
            id,
            failed_transactions: HashMap::new(),
            failed_transaction_log,
            transaction_log,
        }
    }

    pub fn retry(&mut self, id: i32) -> bool {
        if self.failed_transactions.contains_key(&id) {
            let transaction = self
                .failed_transactions
                .get(&id)
                .unwrap_or_else(|| panic!("ALGLOBO <{}>: INTERNAL ERROR", self.id))
                .clone();
            if self.connect_and_process_transaction(transaction, TransactionPhase::Init, true) {
                self.failed_transactions
                    .remove_entry(&id)
                    .unwrap_or_else(|| {
                        panic!(
                            "ALGLOBO <{}>: couldn't remove from map transaction {}",
                            self.id, id
                        )
                    });
            }
        } else {
            return false;
        }
        true
    }

    pub fn process(&mut self, ctrlc_event: Arc<Mutex<Receiver<()>>>) -> bool {
        let ctrlc_pressed = Arc::new(Mutex::new(false));
        let ctrlc_pressed_copy = ctrlc_pressed.clone();

        let transaction_parser = Arc::new(Mutex::new(TransactionParser::new(
            "transactions.txt".to_owned(),
        )));

        let leader_election = LeaderElection::new(self.host.clone(), self.port as u32, self.id); //todo get id from somewhere
        let leader_clone = leader_election.clone();
        let _leader_thread = thread::spawn(move || leader_clone.ping_control());

        if leader_election.am_i_leader() {
            self.init_new_leader(transaction_parser.clone());
        }

        loop {
            if ctrlc_event
                .lock()
                .unwrap_or_else(|_| panic!("ALGLOBO <{}>: could adquire ctrlc lock", self.id))
                .try_recv()
                .is_ok()
            {
                //received ctrlc
                *ctrlc_pressed_copy.lock().unwrap_or_else(|_| {
                    panic!("ALGLOBO <{}>: could adquire ctrlc lock", self.id)
                }) = true;
            } else if !leader_election.am_i_leader() {
                println!("WAITING ELECTION");

                leader_election.wait_until_leader_changes();

                if leader_election.is_done() {
                    break;
                }

                if leader_election.am_i_leader() {
                    self.init_new_leader(transaction_parser.clone());
                }
            } else if let Some(transaction) = transaction_parser
                .lock()
                .expect("poisoned!")
                .read_transaction()
            {
                self.connect_and_process_transaction(Arc::new(transaction), TransactionPhase::Init, false);
            } else {
                break; // no more transacitions
            }
        }
        let forced = *ctrlc_pressed
            .lock()
            .unwrap_or_else(|_| panic!("ALGLOBO <{}>: could adquire ctrlc lock", self.id));
        let leader = leader_election.am_i_leader();
        leader_election.close(!forced);
        forced || !leader
    }

    fn init_new_leader(&mut self, transaction_parser: Arc<Mutex<TransactionParser>>) {
        self.transaction_log.init(); //idempotente
        self.failed_transaction_log.init(); //idempotente

        self.retrieve_failed_transactions();

        // Continue with transaction left from last leader
        let last_processed_transaction = TransactionLogParser::new().get_last_transaction();

        if let Some((id, phase)) = last_processed_transaction {
            if let Some(transaction) = transaction_parser
                .lock()
                .expect("poisoned!")
                .seek_transaction(id)
            {
                self.connect_and_process_transaction(Arc::new(transaction), phase, false);
            }
        }
    }

    fn connect_and_process_transaction(
        &mut self,
        transaction: Arc<Transaction>,
        phase: TransactionPhase,
        is_retry: bool
    ) -> bool {
        let hotel_address = format!("localhost:{}", kind_address(ServiceKind::Hotel));
        let bank_address: String = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let airline_address: String = format!("localhost:{}", kind_address(ServiceKind::Airline));
        let mut service_streams = HashMap::new();
        service_streams.insert(
            ServiceKind::Hotel,
            ServiceStream::new(hotel_address),
        );
        service_streams.insert(
            ServiceKind::Bank,
            ServiceStream::new(bank_address),
        );
        service_streams.insert(
            ServiceKind::Airline,
            ServiceStream::new(airline_address),
        );

        self.process_transaction(transaction, service_streams, phase, is_retry)
    }

    fn process_transaction(
        &mut self,
        transaction: Arc<Transaction>,
        service_streams: HashMap<ServiceKind, ServiceStream>,
        phase: TransactionPhase,
        is_retry: bool
    ) -> bool {
        match phase {
            TransactionPhase::Init => {
                self.transaction_log
                    .write_line(format!("INIT {} alglobo <{}>", transaction.id, self.id));
                let message_kind = if is_retry { MessageKind::Retry } else { MessageKind::Transaction };                
                let responses = self.process_operations(
                    transaction.clone(),
                    message_kind,
                    &service_streams,
                );

                if responses.contains(&MessageKind::Rejection) {
                    self.process_transaction(transaction, service_streams, TransactionPhase::Abort, is_retry)
                } else {
                    self.process_transaction(transaction, service_streams, TransactionPhase::Commit, is_retry)
                }
            }
            TransactionPhase::Abort => {
                let id = transaction.id;
                let transaction_cpy = transaction.clone();
                let transaction_cpy2 = transaction.clone();

                self.transaction_log
                    .write_line(format!("ABORT {} alglobo <{}>", id, self.id));
                self.failed_transaction_log.log_transaction(transaction);
                self.failed_transactions
                    .entry(id)
                    .or_insert_with(|| transaction_cpy);
                self.process_operations(transaction_cpy2, MessageKind::Rejection, &service_streams);
                false
            }
            TransactionPhase::Commit => {
                self.transaction_log
                    .write_line(format!("COMMIT {} alglobo <{}>", transaction.id, self.id));
                self.process_operations(transaction, MessageKind::Confirmation, &service_streams);
                true
            }
        }
    }

    fn process_operations(
        &mut self,
        transaction: Arc<Transaction>,
        kind: MessageKind,
        service_streams: &HashMap<ServiceKind, ServiceStream>,
    ) -> Vec<MessageKind> {
        let mut loglist = vec![];
        for operation in &transaction.operations {
            let body = MessageBody::new(
                transaction.id,
                operation.service,
                operation.amount as i32,
                0,
            );
            let message = Message::new(kind, body);
            match service_streams.get_key_value(&operation.service) {
                Some(entry) => {
                    let mut service_stream = entry.1.clone();
                    if !service_stream.connect_n_send(message) { //asume rejection
                        loglist.push(MessageKind::Rejection);
                        return loglist;
                    }

                    match entry.1.connect_n_recv() {
                        Some(buffer) => {
                            if !buffer.is_empty() {
                                let incoming_message = deserialize(buffer);
                                loglist.push(incoming_message.kind);
                            }
                        },
                        None => { //asume rejection
                            loglist.push(MessageKind::Rejection);
                            return loglist;
                        }
                    }
                }
                None => println!(
                    "ALGLOBO <{}>: service {} not found",
                    self.id, &operation.service
                ),
            }
        }
        loglist
    }

    pub fn retrieve_failed_transactions(&mut self) {
        let mut transaction_parser =
            TransactionParser::new("failed_transactions_log.txt".to_owned());

        while let Some(transaction) = transaction_parser.read_transaction() {
            self.failed_transactions
                .entry(transaction.id)
                .or_insert_with(|| Arc::new(transaction));
        }
    }

    pub fn show_failed_transactions(&self) {
        if self.failed_transactions.is_empty() {
            println!("no failed transactions");
        } else {
            for (key, value) in &self.failed_transactions {
                println!("[TRANSCACTION {}] {}", key, value);
            }
        }
    }
}
