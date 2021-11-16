#[derive(Copy, Clone)]
pub enum ServiceKind {
    Hotel,
    Airline,
    Bank,
}

pub fn parse_kind(raw_kind: String) -> Result<ServiceKind, &'static str> {
    match raw_kind.as_ref() {
        "hotel" => Ok(ServiceKind::Hotel),
        "airline" => Ok(ServiceKind::Airline),
        "bank" => Ok(ServiceKind::Bank),
        _ => Err("Error parsing service kind"), //TODO no se que hacer en este caso
    }
}

pub fn kind_address(kind: ServiceKind) -> i32 {
    match kind {
        ServiceKind::Hotel => 8001,
        ServiceKind::Airline => 8002,
        ServiceKind::Bank => 8003,
    }
}