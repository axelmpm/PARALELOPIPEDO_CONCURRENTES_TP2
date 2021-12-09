use crate::operation::Operation;

pub struct Transaction {
  pub id: i32,
  pub operations: Vec<Operation>,
}

impl Transaction {
  pub fn new(id: i32, operations: Vec<Operation>) -> Transaction {
    Transaction {
      id,
      operations
    }
  }
}