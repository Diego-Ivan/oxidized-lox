use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::{Token, TokenType};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct InterpreterError<'a> {
    error_type: InterpreterErrorType<'a>,
    token: &'a Token,
}

impl<'a> Display for InterpreterError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let err_message = match &self.error_type {
            InterpreterErrorType::WrongUnaryOperands(op, t) => {
                format!("The unary operation {op:?} is not valid over token of type: {t}")
            }
            InterpreterErrorType::DivisionByZero => String::from("Division by zero"),
            InterpreterErrorType::WrongBinaryOperands(t1, op, t2) => {
                format!("Operation of type: {op:?} cannot be applied over operands of types {t1:?} and {t2:?}")
            }
        };

        write!(f, "{err_message}\n[line {}]", self.token.line())
    }
}

impl<'a> std::error::Error for InterpreterError<'a> {}

#[derive(Debug)]
pub enum InterpreterErrorType<'a> {
    WrongUnaryOperands(&'a TokenType, LoxType),
    WrongBinaryOperands(LoxType, &'a TokenType, LoxType),
    DivisionByZero,
}

pub type InterpreterResult<'a, T> = Result<T, InterpreterError<'a>>;

#[derive(Debug)]
pub enum LoxType {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Display for LoxType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(str) => write!(f, "\"{str}\""),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret<'a>(&'a self, statements: &'a [Statement]) -> InterpreterResult<()> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement<'a>(&'a self, statement: &'a Statement) -> InterpreterResult<'a, ()> {
        match statement {
            Statement::Expression(expr) => {
                self.evaluate(expr)?;
            }
            Statement::Print(expr) => {
                let result = self.evaluate(expr)?;
                println!("{result}");
            }
            Statement::Declaration { name, initializer } => todo!(),
        }
        Ok(())
    }

    fn evaluate<'a>(&'a self, expression: &'a Expression) -> InterpreterResult<'a, LoxType> {
        match expression {
            Expression::True => Ok(LoxType::Boolean(true)),
            Expression::False => Ok(LoxType::Boolean(false)),
            Expression::Number(num) => Ok(LoxType::Number(*num)),
            Expression::String(str) => Ok(LoxType::String(str.to_string())),
            Expression::Nil => Ok(LoxType::Nil),
            Expression::Grouping(expr) => self.evaluate(expr),
            Expression::Unary(token, expression) => self.evaluate_unary(token, expression),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expression::Var(_name) => todo!(),
        }
    }

    fn evaluate_unary<'a>(
        &'a self,
        token: &'a Token,
        expression: &'a Expression,
    ) -> InterpreterResult<'a, LoxType> {
        match (token.token_type(), self.evaluate(expression)?) {
            /* Numerical negation */
            (TokenType::Minus, LoxType::Number(num)) => Ok(LoxType::Number(-num)),

            /* Boolean negation */
            (TokenType::Bang, LoxType::Boolean(value)) => Ok(LoxType::Boolean(!value)),

            /* nil will be considered a falsy value */
            (TokenType::Bang, LoxType::Nil) => Ok(LoxType::Boolean(true)),
            /* Zero is a falsy value */
            (TokenType::Bang, LoxType::Number(0.0)) => Ok(LoxType::Boolean(true)),
            /* Any other number is truthy */
            (TokenType::Bang, LoxType::Number(_)) => Ok(LoxType::Boolean(false)),
            (op, expr) => Err(InterpreterError {
                error_type: InterpreterErrorType::WrongUnaryOperands(op, expr),
                token,
            }),
        }
    }

    fn evaluate_binary<'a>(
        &'a self,
        first_operand: &'a Expression,
        operator: &'a Token,
        second_operand: &'a Expression,
    ) -> InterpreterResult<'a, LoxType> {
        match (
            self.evaluate(first_operand)?,
            operator.token_type(),
            self.evaluate(second_operand)?,
        ) {
            /* Algebraic operations */
            (LoxType::Number(a), TokenType::Plus, LoxType::Number(b)) => Ok(LoxType::Number(a + b)),
            (LoxType::Number(a), TokenType::Minus, LoxType::Number(b)) => {
                Ok(LoxType::Number(a - b))
            }
            (LoxType::Number(a), TokenType::Star, LoxType::Number(b)) => Ok(LoxType::Number(a * b)),

            /* Handle division by zero */
            (LoxType::Number(a), TokenType::Slash, LoxType::Number(0f64)) => {
                Err(InterpreterError {
                    error_type: InterpreterErrorType::DivisionByZero,
                    token: operator,
                })
            }
            (LoxType::Number(a), TokenType::Slash, LoxType::Number(b)) => {
                Ok(LoxType::Number(a / b))
            }

            /* Logical comparisons */
            (LoxType::Number(a), TokenType::EqualEqual, LoxType::Number(b)) => {
                Ok(LoxType::Boolean(a == b))
            }
            (LoxType::Number(a), TokenType::GreaterEqual, LoxType::Number(b)) => {
                Ok(LoxType::Boolean(a >= b))
            }
            (LoxType::Number(a), TokenType::Greater, LoxType::Number(b)) => {
                Ok(LoxType::Boolean(a > b))
            }
            (LoxType::Number(a), TokenType::LessEqual, LoxType::Number(b)) => {
                Ok(LoxType::Boolean(a <= b))
            }
            (LoxType::Number(a), TokenType::Less, LoxType::Number(b)) => {
                Ok(LoxType::Boolean(a < b))
            }

            /* Boolean operations */
            (LoxType::Boolean(a), TokenType::Or, LoxType::Boolean(b)) => {
                Ok(LoxType::Boolean(a || b))
            }
            (LoxType::Boolean(a), TokenType::And, LoxType::Boolean(b)) => {
                Ok(LoxType::Boolean(a && b))
            }

            /* String operations */
            (LoxType::String(mut s1), TokenType::Plus, LoxType::String(s2)) => {
                s1.push_str(&s2);
                Ok(LoxType::String(s1))
            }
            (LoxType::String(s1), TokenType::Plus, any) => {
                Ok(LoxType::String(format!("{s1}{any}")))
            }

            /* Any other invalid operation will be handled here. */
            (t1, op, t2) => Err(InterpreterError {
                token: operator,
                error_type: InterpreterErrorType::WrongBinaryOperands(t1, op, t2),
            }),
        }
    }
}
