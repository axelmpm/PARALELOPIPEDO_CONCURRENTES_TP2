use std::net::TcpStream;
use crate::message::Message;
use crate::message_kind::MessageKind;
use std::io::{BufReader, BufRead, Error};
use std::time::Duration;

pub(crate) struct ServiceStream{
    pub addr: String
}

impl ServiceStream {
    pub fn new(addr: String) -> ServiceStream{
        return ServiceStream{
            addr
        }
    }

    pub fn connect_n_send(&self, msg: Message) -> bool{
        let stream = self.connect();
        if stream == None{
            false
        }
        match stream.write_all(msg.serialize().as_bytes()) {
            Ok(_) => {
                true
            }
            Err(_) => {
                false
            }
        }
    }

    pub fn connect_n_recv(&self) -> Option<String> {
        let stream = self.connect();
        if stream == None{
            None
        }
        let mut reader =
            BufReader::new(stream.try_clone().unwrap_or_else(|_| {
                panic!("ALGLOBO <{}>: could not clone stream", self.id)
            }));
        let mut buffer = String::new();
        stream.set_read_timeout(Some(Duration::from_millis(500)));
        match reader.read_line(&mut buffer) {
            Ok(_) => {
                Some(buffer)
            }
            Err(_) => {
                None
            }
        }
    }

    fn connect(&self) -> Option<TcpStream> {
        match TcpStream::connect(self.addr.clone()){
            Ok(stream) => Some(stream),
            Err(_e) => None
        }
    }
}
