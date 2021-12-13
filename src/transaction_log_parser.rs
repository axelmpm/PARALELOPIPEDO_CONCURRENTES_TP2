use std::fs::{File};
use std::io::BufReader;
use std::io::BufRead;

use crate::transaction_phase::{TransactionPhase, decode};
pub struct TransactionLogParser {}

impl TransactionLogParser {
  pub fn new() -> Self {
    TransactionLogParser {}
  }

  pub fn parse_log_line(&self, line: String) -> (i32, TransactionPhase) {
    let v = line.split(' ').collect::<Vec<&str>>();
    let transaction_id = v[0].to_owned().parse::<i32>().unwrap();
    let transaction_phase = decode(v[1].to_owned());
    (transaction_id, transaction_phase)
  }

  pub fn get_last_transaction(&self) -> Option<(i32,TransactionPhase)> {
    let file = BufReader::new(File::open("transaction_log.txt").unwrap());
    let mut lines: Vec<_> = file.lines().map(|line| { line.unwrap() }).collect();
    lines.reverse();
    
    if let Some(line) = lines.first() {
      return Some(self.parse_log_line(line.to_string()))
    };

    None
  }
}