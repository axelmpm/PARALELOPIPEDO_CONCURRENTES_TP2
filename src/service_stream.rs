use std::net::TcpStream;
use crate::message::Message;
use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::io::Write;


#[derive(Clone)]
pub(crate) struct ServiceStream{
    pub addr: String
}

impl ServiceStream {
    pub fn new(addr: String) -> ServiceStream{
        return ServiceStream{
            addr
        }
    }

    pub fn connect_n_send(&mut self, msg: Message, mut stream: TcpStream) -> bool {
        match stream.write_all(msg.serialize().as_bytes()) {
            Ok(_) => {
                true
            }
            Err(_) => {
                return false
            }
        }
    }

    pub fn connect_n_recv(&self, stream: TcpStream) -> Option<String> {
        stream.set_read_timeout(Some(Duration::from_millis(5000))).expect("could not set timeout");
        let mut reader = BufReader::new(stream);
        let mut buffer = String::new();
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                Some(buffer)
            }
            Err(e) => {
                None
            }
        }
    }

    pub fn connect(&self) -> Option<TcpStream> {
        match TcpStream::connect(self.addr.clone()){
            Ok(stream) => Some(stream),
            Err(_e) => None
        }
    }
}
