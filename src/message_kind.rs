use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MessageKind {
    Confirmation,
    Rejection,
    Transaction,
    Ack,
}

pub fn decode(raw_message: String) -> MessageKind {
    match raw_message.as_ref() {
        "confirmation" => MessageKind::Confirmation,
        "rejection" => MessageKind::Rejection,
        "ack" => MessageKind::Ack,
        _ => MessageKind::Transaction,
    }
}

pub fn encode(msg: MessageKind) -> String {
    match msg {
        MessageKind::Confirmation => "confirmation".to_string(),
        MessageKind::Rejection => "rejection".to_string(),
        MessageKind::Ack => "ack".to_string(),
        _ => "".to_string(),
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MessageKind::Confirmation => "confirmation",
                MessageKind::Rejection => "rejection",
                MessageKind::Transaction => "transaction",
                MessageKind::Ack => "ack",
            }
        )
    }
}
