use crate::service_kind::{ServiceKind, kind_address};
use std::net::{TcpStream};
use std::io::Write;

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
        hotel_stream.write_all("AlGobo".as_bytes());
    }
}