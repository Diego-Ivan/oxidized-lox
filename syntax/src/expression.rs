use crate::token::Token;
use std::fmt::{Debug, Formatter, Write};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
    Unary(Token, Box<Expression>),
    Var {
        name: String,
        token: Token,
    },
    Assignment {
        name: String,
        value: Box<Expression>,
        token: Token,
    },
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        paren: Token,
        args: Vec<Expression>,
    },
    Get {
        expression: Box<Expression>,
        token: Token,
    },
    Set {
        name: Token,
        object: Box<Expression>,
        value: Box<Expression>,
    },

    // Literals
    True,
    False,
    Number(ordered_float::OrderedFloat<f64>),
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
            Expression::Var { name, token: _ } => write!(f, "Var({name})"),
            Expression::Assignment {
                name: _,
                value,
                token: _,
            } => write!(f, "Assign(name = {value:?})"),
            Expression::Or { left, right } => {
                write!(f, "({left:?}) || ({right:?})")
            }
            Expression::And { left, right } => {
                write!(f, "({left:?}) && ({right:?})")
            }
            Expression::Call {
                callee,
                paren: _,
                args,
            } => {
                write!(f, "call (callee: {callee:?}, args: {args:?})")
            }
            Expression::Get { expression, token } => {
                write!(f, "get(expr: {expression:?}, name: {})", token.lexeme())
            }
            Expression::Set {
                name,
                object,
                value,
            } => {
                write!(f, "set(name: {name}, object: {object:?}, value: {value:?})")
            }
        }
    }
}
