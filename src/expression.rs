use crate::token::Token;
use std::fmt::{Debug, Formatter, Write};

pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Unary(Token, Box<Expression>),
    Var(String),

    // Literals
    True,
    False,
    Number(f64),
    String(String),
    Nil,
}

fn parenthesize(
    f: &mut Formatter<'_>,
    name: &str,
    expressions: &[&Expression],
) -> std::fmt::Result {
    f.write_char('(')?;
    f.write_str(name)?;

    for expr in expressions {
        f.write_char(' ')?;
        write!(f, "{expr:?}")?;
    }
    f.write_char(')')?;

    Ok(())
}

impl Debug for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::True => f.write_str("true"),
            Expression::False => f.write_str("false"),
            Expression::Nil => f.write_str("nil"),
            Expression::Number(num) => f.write_str(&num.to_string()),
            Expression::String(str) => f.write_str(str),
            Expression::Binary {
                left,
                operator,
                right,
            } => parenthesize(f, operator.lexeme(), &[left, right]),
            Expression::Grouping(expr) => parenthesize(f, "group", &[expr]),
            Expression::Unary(token, expr) => parenthesize(f, token.lexeme(), &[expr]),
            Expression::Var(name) => write!(f, "Var({name})"),
        }
    }
}
