use std::fs::OpenOptions;
use std::io::prelude::*;
use std::fs::File;
use std::time::Instant;
use crate::message_body::MessageBody;
use crate::transaction::Transaction;
use std::sync::Arc;


pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(log_file_name: String) -> Logger {

        let file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(String::from(log_file_name))
            .expect("LOGGER: Couldn't open log file");

        Logger {file }
    }
    pub fn log(&mut self, message_body: MessageBody) {
        writeln!(
            self.file,
            "{} :: {:?}",
            message_body,
            Instant::now()

        )
        .expect("LOGGER: Couldn't log to file");
    }

    pub fn write_line(&mut self, line: String) {
        writeln!(
            self.file,
            "{} :: {:?}",
            line,
            Instant::now()

        )
        .expect("LOGGER: Couldn't log to file");

        //println!("{}", line);
    }

    pub fn log_transaction(&mut self, transaction: Arc<Transaction>) {
        let total = transaction.operations.len();
        for operation in &transaction.operations{
            self.write_line(format!("{},{},{},{}", transaction.id, operation.service, operation.amount, total));
        }
    }
}