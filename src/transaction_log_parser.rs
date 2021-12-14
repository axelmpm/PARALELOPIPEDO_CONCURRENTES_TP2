use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;

use crate::transaction_phase::{decode, TransactionPhase};
pub struct TransactionLogParser {}

impl TransactionLogParser {
    pub fn new() -> Self {
        TransactionLogParser {}
    }

    pub fn parse_log_line(&self, line: String) -> (i32, TransactionPhase) {
        let v = line.split(' ').collect::<Vec<&str>>();
        let transaction_id = v[1].to_owned().parse::<i32>().unwrap();
        let transaction_phase = decode(v[1].to_owned());
        (transaction_id, transaction_phase)
    }

    pub fn get_last_transaction(&self) -> Option<(i32, TransactionPhase)> {
        let mut file = BufReader::new(File::open("transaction_log.txt").unwrap());
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut lines: Vec<_> = file.lines().map(|line| line.unwrap()).collect();

        if let Some(line) = lines.last() {
            return Some(self.parse_log_line(line.to_string()));
        };

        None
    }
}
