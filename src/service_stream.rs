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

    pub fn connect_n_send(&mut self, msg: Message) -> bool{
        if let Some(mut stream) = self.connect(){
            match stream.write_all(msg.serialize().as_bytes()) {
                Ok(_) => {
                    return true;
                }
                Err(_) => {
                    return false;
                }
            }
        }
        false
    }

    pub fn connect_n_recv(&self) -> Option<String> {
        if let Some(stream) = self.connect() {
            stream.set_read_timeout(Some(Duration::from_millis(500))).expect("could not set timeout");
            let mut reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| {
                    panic!("ALGLOBO: could not clone stream")
                }));
            let mut buffer = String::new();
            match reader.read_line(&mut buffer) {
                Ok(_) => {
                    return Some(buffer);
                }
                Err(_) => {
                    return None;
                }
            }
        }
        None
    }

    fn connect(&self) -> Option<TcpStream> {
        match TcpStream::connect(self.addr.clone()){
            Ok(stream) => Some(stream),
            Err(_e) => None
        }
    }
}
