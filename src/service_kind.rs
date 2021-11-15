#[derive(Copy, Clone)]
pub enum ServiceKind {
    Hotel,
    Airline,
    Bank,
}

pub fn kind_address(kind: ServiceKind) -> u32 {
    match kind {
        ServiceKind::Hotel => 8001,
        ServiceKind::Airline => 8002,
        ServiceKind::Bank => 8003,
    }
}