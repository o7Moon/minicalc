#[derive(Clone)]
pub enum NumberBase {
    Decimal,
    Binary,
    Hexadecimal,
}

impl NumberBase {
    pub fn place_value(&self) -> u32 {
        match self {
            NumberBase::Binary => 2,
            NumberBase::Decimal => 10,
            NumberBase::Hexadecimal => 16,
        }
    }
}