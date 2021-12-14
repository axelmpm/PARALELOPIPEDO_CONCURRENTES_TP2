use crate::message_body::MessageBody;
use crate::transaction::Transaction;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Arc;
use std::time::Instant;

pub struct Logger {
    file_name: String,
    file: File,
    inited: bool,
}

impl Logger {
    pub fn new(log_file_name: String) -> Logger {
        let file_name = log_file_name;
        let file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(String::from("dummy.txt"))
            .expect("LOGGER: Couldn't open log file");
        let inited = false;

        Logger {
            file_name,
            file,
            inited,
        }
    }

    pub fn init(&mut self) {
        if !self.inited {
            println!("initing logger");
            self.file = OpenOptions::new()
                //.truncate(true)
                .create(true)
                .write(true)
                .append(true)
                .open(self.file_name.clone())
                .expect("LOGGER: Couldn't open log file");

            self.inited = true;
        }
    }

    pub fn log(&mut self, message_body: MessageBody) {
        writeln!(self.file, "{} :: {:?}", message_body, Instant::now())
            .expect("LOGGER: Couldn't log to file");
    }

    fn _write_line(&mut self, line: String, with_time: bool) {
        if with_time {
            writeln!(self.file, "{} :: {:?}", line, Instant::now())
                .expect("LOGGER: Couldn't log to file");
        } else {
            writeln!(self.file, "{}", line,).expect("LOGGER: Couldn't log to file");
        }
    }

    pub fn write_line(&mut self, line: String) {
        println!("{}", &line);
        self._write_line(line, true);
    }

    pub fn quiet_write_line(&mut self, line: String) {
        self._write_line(line, false);
    }

    pub fn log_transaction(&mut self, transaction: Arc<Transaction>) {
        let total = transaction.operations.len();
        for operation in &transaction.operations {
            self.quiet_write_line(format!(
                "{},{},{},{}",
                transaction.id, operation.service, operation.amount, total
            ));
        }
    }
}
