use std::boxed::Box;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;

use crate::operation::parse_operation;
use crate::transaction::Transaction;
pub struct TransactionParser {
    lines: Box<dyn Iterator<Item = String>>,
}

impl TransactionParser {
    pub fn new(path: String) -> Self {
        let mut file = File::open(path).expect("Problem opening file");
        file.seek(SeekFrom::Start(0))
            .unwrap_or_else(|_| panic!("TRANSACTION PARSER: INTERNAL ERRROR"));
        let reader = BufReader::new(file);
        let lines = Box::new(reader.lines().flatten());
        TransactionParser { lines }
    }

    pub fn seek_transaction(&mut self, id: i32) -> Option<Transaction> {
        while let Some(tx) = self.read_transaction() {
            if tx.id == id {
                return Some(tx);
            }
        }
        None
    }

    pub fn read_transaction(&mut self) -> Option<Transaction> {
        let mut operations = vec![];
        let total_operations;
        let transaction_id;
        if let Some(line) = self.lines.next() {
            operations.push(parse_operation(line.clone()));
            let s = line.split(',').collect::<Vec<&str>>();
            transaction_id = s
                .first()
                .unwrap_or_else(|| panic!("TRANSACTION PARSER: INTERNAL ERRROR"))
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("TRANSACTION PARSER: INTERNAL ERRROR"));
            total_operations = s
                .last()
                .unwrap_or_else(|| panic!("TRANSACTION PARSER: INTERNAL ERRROR"))
                .parse::<i32>()
                .unwrap_or_else(|_| panic!("TRANSACTION PARSER: INTERNAL ERRROR"));
        } else {
            return None;
        }

        for _ in 1..total_operations {
            let line = self
                .lines
                .next()
                .unwrap_or_else(|| panic!("TRANSACTION PARSER: INTERNAL ERRROR"));
            operations.push(parse_operation(line));
        }

        Some(Transaction::new(transaction_id, operations))
    }
}
