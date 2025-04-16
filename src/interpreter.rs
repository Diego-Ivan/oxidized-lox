mod environment;
mod error;
pub mod statement;
mod value;

use crate::expression::Expression;
use crate::interpreter::environment::Environment;
use crate::token::{Token, TokenType};
pub use error::*;
pub use statement::Statement;
use std::cell::RefCell;
use std::rc::Rc;
pub use value::LoxValue;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret<'a>(&'a self, statements: &'a [Statement]) -> InterpreterResult<'a, ()> {
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
            Statement::Declaration { name, initializer } => {
                let initial = match initializer.as_ref() {
                    Some(initializer) => self.evaluate(initializer)?,
                    None => LoxValue::Nil,
                };
                let mut env = self.environment.borrow_mut();
                env.define(name.to_string(), initial);
            }
        }
        Ok(())
    }

    fn evaluate<'a>(&'a self, expression: &'a Expression) -> InterpreterResult<'a, LoxValue> {
        match expression {
            Expression::True => Ok(LoxValue::Boolean(true)),
            Expression::False => Ok(LoxValue::Boolean(false)),
            Expression::Number(num) => Ok(LoxValue::Number(*num)),
            Expression::String(str) => Ok(LoxValue::String(str.to_string())),
            Expression::Nil => Ok(LoxValue::Nil),
            Expression::Grouping(expr) => self.evaluate(expr),
            Expression::Unary(token, expression) => self.evaluate_unary(token, expression),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expression::Var { name, token } => {
                let env = self.environment.borrow();

                let value = match env.get(name) {
                    Some(value) => value,
                    None => {
                        return Err(InterpreterError {
                            error_type: InterpreterErrorType::UndefinedVariable(name.to_string()),
                            token,
                        })
                    }
                };
                Ok(value.clone())
            }
            Expression::Assignment { name, value, token } => {
                let mut env = self.environment.borrow_mut();
                let value = self.evaluate(value)?;
                if !env.set(name.clone(), value.clone()) {
                    return Err(InterpreterError {
                        error_type: InterpreterErrorType::UndefinedVariable(name.clone()),
                        token,
                    });
                }
                Ok(value)
            }
        }
    }

    fn evaluate_unary<'a>(
        &'a self,
        token: &'a Token,
        expression: &'a Expression,
    ) -> InterpreterResult<'a, LoxValue> {
        match (token.token_type(), self.evaluate(expression)?) {
            /* Numerical negation */
            (TokenType::Minus, LoxValue::Number(num)) => Ok(LoxValue::Number(-num)),

            /* Boolean negation */
            (TokenType::Bang, LoxValue::Boolean(value)) => Ok(LoxValue::Boolean(!value)),

            /* nil will be considered a falsy value */
            (TokenType::Bang, LoxValue::Nil) => Ok(LoxValue::Boolean(true)),
            /* Zero is a falsy value */
            (TokenType::Bang, LoxValue::Number(0.0)) => Ok(LoxValue::Boolean(true)),
            /* Any other number is truthy */
            (TokenType::Bang, LoxValue::Number(_)) => Ok(LoxValue::Boolean(false)),
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
    ) -> InterpreterResult<'a, LoxValue> {
        match (
            self.evaluate(first_operand)?,
            operator.token_type(),
            self.evaluate(second_operand)?,
        ) {
            /* Algebraic operations */
            (LoxValue::Number(a), TokenType::Plus, LoxValue::Number(b)) => {
                Ok(LoxValue::Number(a + b))
            }
            (LoxValue::Number(a), TokenType::Minus, LoxValue::Number(b)) => {
                Ok(LoxValue::Number(a - b))
            }
            (LoxValue::Number(a), TokenType::Star, LoxValue::Number(b)) => {
                Ok(LoxValue::Number(a * b))
            }

            /* Handle division by zero */
            (LoxValue::Number(_), TokenType::Slash, LoxValue::Number(0f64)) => {
                Err(InterpreterError {
                    error_type: InterpreterErrorType::DivisionByZero,
                    token: operator,
                })
            }
            (LoxValue::Number(a), TokenType::Slash, LoxValue::Number(b)) => {
                Ok(LoxValue::Number(a / b))
            }

            /* Logical comparisons */
            (LoxValue::Number(a), TokenType::EqualEqual, LoxValue::Number(b)) => {
                Ok(LoxValue::Boolean(a == b))
            }
            (LoxValue::Number(a), TokenType::GreaterEqual, LoxValue::Number(b)) => {
                Ok(LoxValue::Boolean(a >= b))
            }
            (LoxValue::Number(a), TokenType::Greater, LoxValue::Number(b)) => {
                Ok(LoxValue::Boolean(a > b))
            }
            (LoxValue::Number(a), TokenType::LessEqual, LoxValue::Number(b)) => {
                Ok(LoxValue::Boolean(a <= b))
            }
            (LoxValue::Number(a), TokenType::Less, LoxValue::Number(b)) => {
                Ok(LoxValue::Boolean(a < b))
            }

            /* Boolean operations */
            (LoxValue::Boolean(a), TokenType::Or, LoxValue::Boolean(b)) => {
                Ok(LoxValue::Boolean(a || b))
            }
            (LoxValue::Boolean(a), TokenType::And, LoxValue::Boolean(b)) => {
                Ok(LoxValue::Boolean(a && b))
            }

            /* String operations */
            (LoxValue::String(mut s1), TokenType::Plus, LoxValue::String(s2)) => {
                s1.push_str(&s2);
                Ok(LoxValue::String(s1))
            }
            (LoxValue::String(s1), TokenType::Plus, any) => {
                Ok(LoxValue::String(format!("{s1}{any}")))
            }

            /* Any other invalid operation will be handled here. */
            (t1, op, t2) => Err(InterpreterError {
                token: operator,
                error_type: InterpreterErrorType::WrongBinaryOperands(t1, op, t2),
            }),
        }
    }
}
