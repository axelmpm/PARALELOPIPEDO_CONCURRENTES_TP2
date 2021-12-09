use std::fs::{File};
use std::boxed::Box;
use std::io::BufReader;
use std::io::BufRead;

use crate::operation::{parse_operation};
use crate::transaction::Transaction;
pub struct TransactionParser {
  lines: Box<dyn Iterator<Item = String>>
}

impl TransactionParser {
  pub fn new(path: String) -> Self {
    let file = File::open(path).expect("Problem opening file");
    let reader = BufReader::new(file);
    let lines = Box::new(reader.lines().flatten());
    TransactionParser {
      lines
    }
  }

  pub fn read_transaction(&mut self) -> Option<Transaction> {
    let mut operations = vec![];
    let total_operations;
    let transaction_id ;
    if let Some(line) = self.lines.next() {
      operations.push(parse_operation(line.clone()));
      let s = line.split(',').collect::<Vec<&str>>();
      transaction_id = s.first().unwrap().parse::<i32>().unwrap();
      total_operations = s.last().unwrap().parse::<i32>().unwrap();
    } else {
      return None
    }

    for _ in 1..total_operations {
      let line = self.lines.next().unwrap();
      operations.push(parse_operation(line));
    }
    
    Some(Transaction::new(transaction_id, operations))
  }
}