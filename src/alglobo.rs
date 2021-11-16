use crate::service_kind::{ServiceKind, kind_address};
use crate::transaction_step::{TransactionStep, transaction_parser};
use std::net::{TcpStream};
use std::io::Write;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;


pub struct Alglobo {
    host: String,
    port: i32,
}

impl Alglobo {

    pub fn new(host: String, port: i32) -> Self {
        return Alglobo{host, port};
    }

    pub fn run(&self) {

        let hotel_address = format!("localhost:{}", kind_address(ServiceKind::Hotel));
        let mut hotel_stream = TcpStream::connect(hotel_address).unwrap();
        /*
        let airline_address = format!("localhost:{}", kind_address(ServiceKind::Airline));
        let mut airline_stream = TcpStream::connect(airline_address).unwrap();

        let bank_address = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let mut bank_stream = TcpStream::connect(bank_address).unwrap();*/

        //parsear archivo transacciones

        let f = File::open("transactions.txt");
        let file = match f {
            Ok(file) => file,
            Err(error) => {
                println!("problem opening file: {:?}", error);
                return;
            }
        };
        let reader = BufReader::new(file);

        for line in reader.lines().flatten() {

            let transaction_step = transaction_parser(line);
        }

        hotel_stream.write_all("AlGobo1\n".as_bytes());
        hotel_stream.write_all("AlGobo2\n".as_bytes());
        hotel_stream.write_all("AlGobo3\n".as_bytes());
    }
}