#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operation {
    pub fn char(&self) -> &str {
        match self {
            Operation::Add => "+",
            Operation::Sub => "-",
            Operation::Mul => "*",
            Operation::Div => "/",
            Operation::Mod => "%",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => {Some(Operation::Add)},
            "-" => {Some(Operation::Sub)},
            "*" => {Some(Operation::Mul)},
            "/" => {Some(Operation::Div)},
            "%" => {Some(Operation::Mod)},
            _ => {None}
        }
    }
}