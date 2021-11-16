
use crate::message_kind::MessageKind;
use crate::message_kind::decode;
use crate::transaction_step::TransactionStep;

pub struct Message {
    message_kind: MessageKind,
    transaction: TransactionStep //quizas?
}