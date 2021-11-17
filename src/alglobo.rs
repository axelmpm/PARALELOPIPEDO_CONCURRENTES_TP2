use crate::service_kind::{ServiceKind, kind_address};
use crate::message_body::body_parser;
use crate::message::{Message, deserialize};
use crate::message_kind::MessageKind;
use crate::logger::Logger;
use std::net::{TcpStream};
use std::io::{Write, Read};
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
        let mut hotel_reader = BufReader::new(hotel_stream.try_clone().expect("could not clone stream"));
        /*
        let airline_address = format!("localhost:{}", kind_address(ServiceKind::Airline));
        let mut airline_stream = TcpStream::connect(airline_address).unwrap();

        let bank_address = format!("localhost:{}", kind_address(ServiceKind::Bank));
        let mut bank_stream = TcpStream::connect(bank_address).unwrap();*/

        //parsear archivo transacciones

        let file = File::open("transactions.txt").expect("Problem opening file");
        let mut rejected = Logger::new("rejections.txt".to_owned());
        let mut accepted = Logger::new("accepted.txt".to_owned());
        
        let reader = BufReader::new(file);
        

        for line in reader.lines().flatten() {
            println!("linea print {}", line);
            let body = body_parser(line);
            
            hotel_stream.write_all(Message::new(MessageKind::Transaction, body).serialize().as_bytes()); //TODO handelear error y reconectarse
            
            let mut buffer = String::new();
            hotel_reader.read_line(&mut buffer);
            let incoming_message = deserialize(buffer);

            match incoming_message.kind.clone() {
                MessageKind::Confirmation => {
                    accepted.log(incoming_message.body);
                },
    
                MessageKind::Rejection => {
                    rejected.log(incoming_message.body);
                },
    
                _ => {},
            }
        
        }

        hotel_stream.write_all("AlGobo1\n".as_bytes());
        hotel_stream.write_all("AlGobo2\n".as_bytes());
        hotel_stream.write_all("AlGobo3\n".as_bytes());
    }
}