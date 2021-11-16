
use crate::service_kind::{ServiceKind, parse_kind};

pub struct TransactionStep {
    id: i32,
    service: ServiceKind,
    amount: i32,
    total: i32,
}

impl TransactionStep {
    pub fn new(id: i32, service: ServiceKind, amount: i32, total: i32) -> TransactionStep {
      TransactionStep {
        id,
        service,
        amount,
        total,
      }
    }
}

pub fn transaction_parser(line: String) -> TransactionStep {

    let params = line.split(',').collect::<Vec<&str>>();
    let id = params[0].parse::<i32>().unwrap();
    let service = parse_kind(params[1].to_string()).unwrap();
    let amount = params[2].parse::<i32>().unwrap();
    let total = params[3].parse::<i32>().unwrap();
    return TransactionStep::new(id, service, amount, total);
}