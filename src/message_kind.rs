use std::fmt;


#[derive(Copy, Clone)]
pub enum MessageKind {
    Confirmation,
    Rejection,
    Transaction,
} 

pub fn decode(raw_message: String) -> MessageKind {
    match raw_message.as_ref() {
        "confirmation" => MessageKind::Confirmation,
        "rejection" => MessageKind::Rejection,
        _ => MessageKind::Transaction,
    }
}

pub fn encode(msg: MessageKind) -> String {
    match msg {
        MessageKind::Confirmation => "confirmation".to_string() ,
        MessageKind::Rejection => "rejection".to_string() ,
        _ => "".to_string() ,
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            MessageKind::Confirmation => "confirmation",
            MessageKind::Rejection => "rejection",
            MessageKind::Transaction => "transaction",
        })
    }
}