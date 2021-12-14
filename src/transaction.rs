use crate::operation::Operation;
use std::fmt;

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

impl fmt::Display for Transaction {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{},{:?}",self.id, self.operations)
  }
}