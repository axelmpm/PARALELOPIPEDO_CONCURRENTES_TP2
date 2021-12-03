use crate::operation::Operation;

pub struct Transaction {
  pub id: String,
  pub operations: Vec<Operation>,
}

impl Transaction {
  pub fn new(id: String, operations: Vec<Operation>) -> Transaction {
    Transaction {
      id,
      operations
    }
  }
}