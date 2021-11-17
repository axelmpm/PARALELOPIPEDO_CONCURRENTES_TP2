
use crate::message_kind::MessageKind;
use crate::message_kind::decode;
use crate::message_body::{MessageBody, body_parser};

pub struct Message {
    pub message_kind: MessageKind,
    pub body: MessageBody //quizas?
}

impl Message {

    pub fn new(message_kind: MessageKind, body: MessageBody) -> Message{
        return Message{message_kind: message_kind, body: body};
    }

    pub fn serialize(&self) -> String{
        format!("{}\n", self.body)
    }
}

pub fn deserialize(raw_message : String ) -> Message{

    let params = raw_message.split(',').collect::<Vec<&str>>();
    let message_kind = decode(params[0].to_string());
    let raw_body = params[1].to_string();
    let body = body_parser(raw_body);

    Message::new(message_kind, body)
}