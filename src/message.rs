
use crate::message_kind::MessageKind;
use crate::message_kind::decode;
use crate::transaction_step::TransactionStep;

pub struct Message {
    message_kind: MessageKind,
    transaction: TransactionStep //quizas?
}

impl Message {

    pub fn new(message_kind: MessageKind, transaction: TransactionStep) -> Message{
        return Message{message_kind: message_kind, transaction: transaction};
    }

    pub fn serialize(&self) -> String{
        format!("{}", self.transaction)
    }
}