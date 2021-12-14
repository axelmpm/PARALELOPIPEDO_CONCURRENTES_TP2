use std::fmt;


#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TransactionPhase {
  Init,
  Abort,
  Commit
}

pub fn decode(raw_message: String) -> TransactionPhase {
  match raw_message.as_ref() {
      "INIT" => TransactionPhase::Init,
      "COMMIT" => TransactionPhase::Commit,
      "ABORT" => TransactionPhase::Abort,
      _ => TransactionPhase::Init
  }
}

pub fn encode(msg: TransactionPhase) -> String {
  match msg {
      TransactionPhase::Init => "INIT".to_string(),
      TransactionPhase::Commit => "COMMIT".to_string(),
      TransactionPhase::Abort => "ABORT".to_string(),
      _ => "".to_string() ,
  }
}

impl fmt::Display for TransactionPhase {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}", encode(*self))
  }
} 