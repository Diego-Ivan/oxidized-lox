mod callable;
mod environment;
mod error;
mod native;
pub mod statement;
mod value;

use crate::expression::Expression;
use crate::interpreter::callable::{Callable, NativeFunc};
use crate::interpreter::environment::Environment;
use crate::interpreter::statement::Block;
use crate::token::{Token, TokenType};
pub use error::*;
pub use statement::Statement;
use std::cell::RefCell;
use std::rc::Rc;
pub use value::LoxValue;

type RcEnvironment = Rc<RefCell<Environment>>;

pub struct Interpreter {
    globals: RcEnvironment,
    environment_stack: RefCell<Vec<RcEnvironment>>,
}

pub enum ControlFlow {
    Normal,
    Return(LoxValue),
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let interpreter = Self {
            environment_stack: RefCell::new(vec![globals.clone()]),
            globals,
        };
        interpreter.load_native_functions();

        interpreter
    }

    pub fn interpret(&self, statements: &[Statement]) -> InterpreterResult<()> {
        for statement in statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&self, statement: &Statement) -> InterpreterResult<ControlFlow> {
        match statement {
            Statement::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(ControlFlow::Normal)
            }
            Statement::Print(expr) => {
                let result = self.evaluate(expr)?;
                println!("{result}");
                Ok(ControlFlow::Normal)
            }
            Statement::VariableDeclaration { name, initializer } => {
                let initial = match initializer.as_ref() {
                    Some(initializer) => self.evaluate(initializer)?,
                    None => LoxValue::Nil,
                };
                let env_stack = self.environment_stack.borrow_mut();
                let mut env = env_stack.last().unwrap().borrow_mut();
                env.define(name.to_string(), initial);

                Ok(ControlFlow::Normal)
            }
            Statement::Block(statements) => {
                let current_env = {
                    let env_stack = self.environment_stack.borrow_mut();
                    env_stack.last().unwrap().clone()
                };

                let enclosure = Environment::new_enclosed(current_env);

                self.execute_block(statements, Rc::new(RefCell::new(enclosure)))
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let result = self.evaluate(condition)?.is_truthy();

                if result {
                    self.execute_statement(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute_statement(else_branch)
                } else {
                    Ok(ControlFlow::Normal)
                }
            }
            Statement::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute_statement(body)?;
                }
                Ok(ControlFlow::Normal)
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                let function = Callable::LoxFunction {
                    name: name.clone(),
                    params: parameters.clone(),
                    block: body.clone(),
                };

                let mut global = self.globals.borrow_mut();
                global.define(name.clone(), LoxValue::Callable(Rc::new(function)));
                Ok(ControlFlow::Normal)
            }
            Statement::Return {
                keyword: _,
                expression,
            } => {
                let value = match expression {
                    Some(expression) => self.evaluate(expression)?,
                    None => LoxValue::Nil,
                };
                Ok(ControlFlow::Return(value))
            }
        }
    }

    fn execute_block(
        &self,
        statements: &[Statement],
        env: Rc<RefCell<Environment>>,
    ) -> InterpreterResult<ControlFlow> {
        for statement in statements {
            {
                let mut env_mut = self.environment_stack.borrow_mut();
                env_mut.push(env.clone());
            }

            let result = self.execute_statement(statement);
            self.environment_stack.borrow_mut().pop();

            match result? {
                ControlFlow::Normal => continue,
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
            }
        }

        Ok(ControlFlow::Normal)
    }

    fn evaluate(&self, expression: &Expression) -> InterpreterResult<LoxValue> {
        match expression {
            Expression::True => Ok(LoxValue::Boolean(true)),
            Expression::False => Ok(LoxValue::Boolean(false)),
            Expression::Number(num) => Ok(LoxValue::Number(*num)),
            Expression::String(str) => Ok(LoxValue::String(Rc::new(str.to_string()))),
            Expression::Nil => Ok(LoxValue::Nil),
            Expression::Grouping(expr) => self.evaluate(expr),
            Expression::Unary(token, expression) => self.evaluate_unary(token, expression),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left, operator, right),
            Expression::Var { name, token } => {
                let env_stack = self.environment_stack.borrow_mut();
                let env = env_stack.last().unwrap().borrow();
                let value = match env.get(name) {
                    Some(value) => value,
                    None => {
                        return Err(InterpreterError {
                            error_type: InterpreterErrorType::UndefinedVariable(name.to_string()),
                            token: token.clone(),
                        })
                    }
                };
                Ok(value.clone())
            }
            Expression::Assignment { name, value, token } => {
                let value = self.evaluate(value)?;
                let env_stack = self.environment_stack.borrow_mut();
                let mut env = env_stack.last().unwrap().borrow_mut();

                if !env.set(name.clone(), value.clone()) {
                    return Err(InterpreterError {
                        error_type: InterpreterErrorType::UndefinedVariable(name.clone()),
                        token: token.clone(),
                    });
                }
                Ok(value)
            }
            Expression::Or { left, right } => {
                let left = self.evaluate(left)?;
                if left.is_truthy() {
                    Ok(left)
                } else {
                    self.evaluate(right)
                }
            }
            Expression::And { left, right } => {
                let left = self.evaluate(left)?;
                if !left.is_truthy() {
                    Ok(left)
                } else {
                    self.evaluate(right)
                }
            }
            Expression::Call {
                callee,
                paren,
                args,
            } => {
                let function = match self.evaluate(callee)? {
                    LoxValue::Callable(callable) => callable,
                    _ => {
                        return Err(InterpreterError {
                            token: paren.clone(),
                            error_type: InterpreterErrorType::NotACallable,
                        })
                    }
                };

                let mut arguments = Vec::new();
                for arg in args {
                    arguments.push(self.evaluate(arg)?);
                }

                match &*function {
                    Callable::Native { func, arity } => {
                        self.evaluate_native(paren, *arity, func, &arguments)
                    }
                    Callable::LoxFunction {
                        name: _,
                        params,
                        block,
                    } => self.evaluate_lox_function(paren, params, arguments, block),
                }
            }
        }
    }

    fn evaluate_lox_function(
        &self,
        token: &Token,
        params: &[Token],
        arguments: Vec<LoxValue>,
        block: &Block,
    ) -> InterpreterResult<LoxValue> {
        let mut function_env =
            Environment::new_enclosed(self.environment_stack.borrow().last().unwrap().clone());

        if params.len() != arguments.len() {
            return Err(InterpreterError {
                error_type: InterpreterErrorType::WrongArity {
                    original: params.len(),
                    user: arguments.len(),
                },
                token: token.clone(),
            });
        }

        for (i, arg) in arguments.into_iter().enumerate() {
            function_env.define(params[i].lexeme().to_string(), arg);
        }

        let value = match self.execute_block(block, Rc::new(RefCell::new(function_env)))? {
            ControlFlow::Normal => LoxValue::Nil,
            ControlFlow::Return(val) => val,
        };

        Ok(value)
    }

    fn evaluate_native(
        &self,
        token: &Token,
        arity: usize,
        func: &NativeFunc,
        arguments: &[LoxValue],
    ) -> InterpreterResult<LoxValue> {
        if arity != arguments.len() {
            return Err(InterpreterError {
                error_type: InterpreterErrorType::WrongArity {
                    original: arity,
                    user: arguments.len(),
                },
                token: token.clone(),
            });
        }

        match func(arguments) {
            Ok(result) => Ok(result),
            Err(e) => Err(InterpreterError {
                token: token.clone(),
                error_type: InterpreterErrorType::Native(e),
            }),
        }
    }

    fn evaluate_unary(
        &self,
        token: &Token,
        expression: &Expression,
    ) -> InterpreterResult<LoxValue> {
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
                error_type: InterpreterErrorType::WrongUnaryOperands(op.clone(), expr),
                token: token.clone(),
            }),
        }
    }

    fn evaluate_binary(
        &self,
        first_operand: &Expression,
        operator: &Token,
        second_operand: &Expression,
    ) -> InterpreterResult<LoxValue> {
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
                    token: operator.clone(),
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

            /* String operations */
            (LoxValue::String(s1), TokenType::Plus, LoxValue::String(s2)) => {
                let mut s1 = s1.to_string();
                s1.push_str(&s2);
                Ok(LoxValue::String(Rc::new(s1)))
            }
            (LoxValue::String(s1), TokenType::Plus, any) => {
                Ok(LoxValue::String(Rc::new(format!("{s1}{any}"))))
            }

            /* Any other invalid operation will be handled here. */
            (t1, op, t2) => Err(InterpreterError {
                token: operator.clone(),
                error_type: InterpreterErrorType::WrongBinaryOperands(t1, op.clone(), t2),
            }),
        }
    }

    fn load_native_functions(&self) {
        let mut _global = self.globals.borrow_mut();

        macro_rules! define_native {
            ($name: literal, $arity: expr, $fun: expr) => {{
                let func = Callable::Native {
                    arity: $arity,
                    func: $fun,
                };
                _global.define(String::from($name), LoxValue::Callable(Rc::new(func)));
            }};
        }

        define_native!("clock", 0, native::clock);
        define_native!("read_line", 0, native::read_line);
        define_native!("random", 2, native::random);
        define_native!("string_to_number", 1, native::string_to_number);
    }
}
