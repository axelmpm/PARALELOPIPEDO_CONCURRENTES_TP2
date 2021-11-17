
use crate::message_kind::MessageKind;
use crate::message_kind::decode;
use crate::message_body::{MessageBody, body_parser};
use std::fmt;

pub struct Message {
    pub kind: MessageKind,
    pub body: MessageBody //quizas?
}

impl Message {

    pub fn new(kind: MessageKind, body: MessageBody) -> Message{
        return Message{kind: kind, body: body};
    }

    pub fn serialize(&self) -> String{
        format!("{},{}\n", self.kind, self.body)
    }
}

pub fn deserialize(raw_message : String ) -> Message{

    println!("error en {}",raw_message);

    let params = raw_message.split(',').collect::<Vec<&str>>();
    let kind = decode(params[0].to_string());
    let raw_body = params[1].to_string();

    println!("kind: {}", kind);
    println!("raw_body: {}", raw_body);
    let body = body_parser(raw_body);

    Message::new(kind, body)
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.kind, self.body)
    }
  }