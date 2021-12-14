use crate::service_kind::{parse_kind, ServiceKind};

#[derive(Debug)]
pub struct Operation {
    pub service: ServiceKind,
    pub amount: u64,
}

impl Operation {
    pub fn new(service: ServiceKind, amount: u64) -> Operation {
        Operation { service, amount }
    }
}

pub fn parse_operation(str: String) -> Operation {
    let s = str.split(',').collect::<Vec<&str>>();
    Operation {
        service: parse_kind(s[1].to_owned())
            .unwrap_or_else(|_| panic!("OPERATION: INTERNAL ERRROR")),
        amount: s[2]
            .to_owned()
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("OPERATION: INTERNAL ERRROR")),
    }
}
