use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum LoxValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(str) => write!(f, "\"{str}\""),
        }
    }
}
