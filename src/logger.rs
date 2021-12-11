use std::fs::OpenOptions;
use std::io::prelude::*;
use std::fs::File;
use std::time::Instant;
use crate::message_body::MessageBody;


pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(log_file_name: String) -> Logger {

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
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

        printnl!("{} ::", line);
    }
}