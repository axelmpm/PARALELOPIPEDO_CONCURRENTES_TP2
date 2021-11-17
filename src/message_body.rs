
use crate::service_kind::{ServiceKind, parse_kind};
use std::fmt;

pub struct MessageBody {
    pub id: i32,
    pub service: ServiceKind,
    pub amount: i32,
    pub total: i32,
}

impl MessageBody {
    pub fn new(id: i32, service: ServiceKind, amount: i32, total: i32) -> MessageBody {
      MessageBody {
        id,
        service,
        amount,
        total,
      }
    }
}

pub fn body_parser(line: String) -> MessageBody {

    let params = line.split(',').collect::<Vec<&str>>();
    println!("BODY PARSER: line = {}, params[0] = {}, params[1] = {}, params[2] = {}, params[3] = {}", line, params[0], params[1], params[2], params[3]);
    let id = params[0].parse::<i32>().unwrap();
    let amount = params[2].parse::<i32>().unwrap();
    let total = params[3].parse::<i32>().unwrap();
    let service = parse_kind(params[1].to_string()).unwrap();
  
    return MessageBody::new(id, service, amount, total);
}

impl fmt::Display for MessageBody {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{},{},{},{}", self.id, self.service, self.amount, self.total)
  }
}