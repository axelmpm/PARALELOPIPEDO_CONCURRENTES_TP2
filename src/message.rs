use crate::message_body::{body_parser, MessageBody};
use crate::message_kind::decode;
use crate::message_kind::MessageKind;
use std::fmt;

pub struct Message {
    pub kind: MessageKind,
    pub body: MessageBody, //quizas?
}

impl Message {
    pub fn new(kind: MessageKind, body: MessageBody) -> Message {
        Message { kind, body }
    }

    pub fn serialize(&self) -> String {
        format!("{}|{},\n", self.kind, self.body)
    }
}

pub fn deserialize(raw_message: String) -> Message {
    let params = raw_message.split('|').collect::<Vec<&str>>();
    //println!("DESERIALIZE: raw_message = {}", raw_message);
    //println!("DESERIALIZE: params[0] = {}, params[1] = {}", params[0], params[1]);
    let body = body_parser(params[1].to_string());
    let kind = decode(params[0].to_string());

    Message::new(kind, body)
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.kind, self.body)
    }
}
