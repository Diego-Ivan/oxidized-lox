mod callable;
mod environment;
mod error;
mod native;
mod value;

use crate::interpreter::callable::{Callable, NativeFunc};
use crate::interpreter::environment::Environment;
use callable::LoxFunction;
pub use error::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use syntax::Expression;
use syntax::statement::Block;
pub use syntax::statement::Statement;
use syntax::token::{Token, TokenType};
use value::Field;
pub use value::LoxValue;

type RcEnvironment = Rc<RefCell<Environment>>;

pub struct Interpreter {
    globals: RcEnvironment,
    environment_stack: RefCell<Vec<RcEnvironment>>,
    locals: RefCell<HashMap<Expression, usize>>,
}

#[must_use]
enum ControlFlow {
    Normal,
    BreakLoop,
    ContinueLoop,
    Return(LoxValue),
}

macro_rules! interpreter_error {
    ($type: expr, $token: expr) => {{
        Err(Box::new(InterpreterError {
            error_type: $type,
            token: $token,
        }))
    }};
}

impl Interpreter {
    pub fn new() -> Self {
        let ref_cell = Rc::new(RefCell::new(Environment::new()));
        let globals = ref_cell;
        let interpreter = Self {
            environment_stack: RefCell::new(vec![globals.clone()]),
            globals,
            locals: RefCell::new(HashMap::new()),
        };
        interpreter.load_native_functions();

        interpreter
    }

    pub fn interpret(&self, statements: &[Statement]) -> InterpreterResult<()> {
        for statement in statements {
            let _ = self.execute_statement(statement, false)?;
        }
        Ok(())
    }

    pub fn resolve(&self, expression: &Expression, depth: usize) {
        let mut locals = self.locals.borrow_mut();
        locals.insert(expression.clone(), depth);
    }

    fn execute_statement(
        &self,
        statement: &Statement,
        inside_loop: bool,
    ) -> InterpreterResult<ControlFlow> {
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

                self.execute_block(statements, Rc::new(RefCell::new(enclosure)), inside_loop)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let result = self.evaluate(condition)?.is_truthy();

                if result {
                    self.execute_statement(then_branch, inside_loop)
                } else if let Some(else_branch) = else_branch {
                    self.execute_statement(else_branch, inside_loop)
                } else {
                    Ok(ControlFlow::Normal)
                }
            }
            Statement::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    match self.execute_statement(body, true)? {
                        ControlFlow::BreakLoop => break,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::ContinueLoop => continue,
                        ControlFlow::Normal => {}
                    };
                }
                Ok(ControlFlow::Normal)
            }
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(initializer) = initializer {
                    let _ = self.execute_statement(initializer, false)?;
                }

                loop {
                    if let Some(condition) = condition {
                        if !self.evaluate(condition)?.is_truthy() {
                            break;
                        }
                    }

                    match self.execute_statement(body, true)? {
                        ControlFlow::Normal => {}
                        ControlFlow::BreakLoop => break,
                        ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
                        ControlFlow::ContinueLoop => {
                            if let Some(increment) = increment {
                                self.evaluate(increment)?;
                            }
                            continue;
                        }
                    };

                    if let Some(increment) = increment {
                        self.evaluate(increment)?;
                    }
                }

                Ok(ControlFlow::Normal)
            }
            Statement::ClassDeclaration { name, methods } => {
                let environment = {
                    let env_stack = self.environment_stack.borrow_mut();
                    env_stack.last().unwrap().clone()
                };

                {
                    let mut environment = environment.borrow_mut();
                    environment.define(name.to_string(), LoxValue::Nil);
                }

                let methods: HashMap<String, Rc<Callable>> = methods
                    .iter()
                    .map(|m| {
                        (
                            m.name.to_string(),
                            Rc::new(Callable::LoxFunction(LoxFunction {
                                closure: environment.clone(),
                                name: m.name.to_string(),
                                params: m.parameters.clone(),
                                block: m.body.clone(),
                            })),
                        )
                    })
                    .collect();

                let class = value::Class::new(name.to_string(), methods);

                let constructor = Callable::Constructor(Rc::new(class));
                environment.borrow_mut().assign_at(
                    name,
                    LoxValue::Callable(Rc::new(constructor)),
                    0,
                );

                Ok(ControlFlow::Normal)
            }
            Statement::FunctionDeclaration(function) => {
                let env_stack = self.environment_stack.borrow();
                let current_env = env_stack.last().unwrap();

                let callable = Callable::LoxFunction(LoxFunction {
                    closure: current_env.clone(),
                    name: function.name.clone(),
                    params: function.parameters.clone(),
                    block: function.body.clone(),
                });

                let mut global = self.globals.borrow_mut();
                global.define(function.name.clone(), LoxValue::Callable(Rc::new(callable)));
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
            Statement::Break { .. } if inside_loop => Ok(ControlFlow::BreakLoop),
            Statement::Continue { .. } if inside_loop => Ok(ControlFlow::ContinueLoop),
            Statement::Break { keyword } | Statement::Continue { keyword } => {
                interpreter_error!(InterpreterErrorType::NotInLoop, keyword.clone())
            }
        }
    }

    fn execute_block(
        &self,
        statements: &[Statement],
        env: Rc<RefCell<Environment>>,
        inside_loop: bool,
    ) -> InterpreterResult<ControlFlow> {
        for statement in statements {
            {
                let mut env_mut = self.environment_stack.borrow_mut();
                env_mut.push(env.clone());
            }

            let result = self.execute_statement(statement, inside_loop);
            self.environment_stack.borrow_mut().pop();

            match result? {
                ControlFlow::Normal => continue,
                ControlFlow::BreakLoop => return Ok(ControlFlow::BreakLoop),
                ControlFlow::ContinueLoop => return Ok(ControlFlow::ContinueLoop),
                ControlFlow::Return(val) => return Ok(ControlFlow::Return(val)),
            }
        }

        Ok(ControlFlow::Normal)
    }

    fn evaluate(&self, expression: &Expression) -> InterpreterResult<LoxValue> {
        match expression {
            Expression::True => Ok(LoxValue::Boolean(true)),
            Expression::False => Ok(LoxValue::Boolean(false)),
            Expression::Number(num) => Ok(LoxValue::Number(**num)),
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
                let value = match self.lookup_variable(name, expression) {
                    Some(value) => value,
                    None => {
                        return interpreter_error!(
                            InterpreterErrorType::UndefinedVariable(name.to_string()),
                            token.clone()
                        );
                    }
                };
                Ok(value.clone())
            }
            Expression::This { keyword } => {
                match self.lookup_variable(keyword.lexeme(), expression) {
                    Some(value) => Ok(value),
                    None => interpreter_error!(
                        InterpreterErrorType::UndefinedVariable(keyword.lexeme().to_string()),
                        keyword.clone()
                    ),
                }
            }
            Expression::Assignment { name, value, token } => {
                let distance = match self.locals.borrow().get(value) {
                    Some(distance) => *distance,
                    None => todo!(),
                };

                let last_env = {
                    let env_stack = self.environment_stack.borrow();
                    env_stack.last().unwrap().clone()
                };

                let value = self.evaluate(value)?;

                if !last_env
                    .borrow_mut()
                    .assign_at(name, value.clone(), distance)
                {
                    return interpreter_error!(
                        InterpreterErrorType::UndefinedVariable(String::from(name)),
                        token.clone()
                    );
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
                        return interpreter_error!(
                            InterpreterErrorType::NotACallable,
                            paren.clone()
                        );
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
                    Callable::LoxFunction(function) => self.evaluate_lox_function(
                        paren,
                        function.closure.clone(),
                        &function.params,
                        arguments,
                        &function.block,
                    ),
                    Callable::Constructor(class) => {
                        if !arguments.is_empty() {
                            return interpreter_error!(
                                InterpreterErrorType::WrongArity {
                                    original: 0,
                                    user: arguments.len()
                                },
                                paren.clone()
                            );
                        }
                        let instance = value::Instance::new(class.clone());
                        Ok(LoxValue::Instance(Rc::new(instance)))
                    }
                }
            }
            Expression::Get { expression, token } => {
                let result = self.evaluate(expression)?;

                match result {
                    LoxValue::Instance(instance) => match instance.get(token.lexeme()) {
                        Field::Value(value) => Ok(value),
                        Field::Method(method) => {
                            Ok(self.bind_method(instance.clone(), method.clone()))
                        }
                        Field::Undefined => interpreter_error!(
                            InterpreterErrorType::NotAProperty {
                                class_name: instance.class_name().to_string(),
                                field: token.lexeme().to_string()
                            },
                            token.clone()
                        ),
                    },
                    _ => {
                        interpreter_error!(
                            InterpreterErrorType::InvalidInstance(token.lexeme().to_string()),
                            token.clone()
                        )
                    }
                }
            }
            Expression::Set {
                name,
                object,
                value,
            } => {
                if let LoxValue::Instance(instance) = self.evaluate(object)? {
                    let value = self.evaluate(value)?;
                    instance.set(name.lexeme(), value.clone());
                    Ok(value)
                } else {
                    // TODO: This should have better formatting
                    interpreter_error!(
                        InterpreterErrorType::InvalidInstance(format!("{object:?}")),
                        name.clone()
                    )
                }
            }
        }
    }

    fn bind_method(&self, instance: Rc<value::Instance>, method: Rc<Callable>) -> LoxValue {
        if let Callable::LoxFunction(function) = &*method {
            let bound_method = Callable::LoxFunction(function.bind(instance));

            LoxValue::Callable(Rc::new(bound_method))
        } else {
            LoxValue::Callable(method)
        }
    }

    fn lookup_variable(&self, name: &str, expression: &Expression) -> Option<LoxValue> {
        let locals = self.locals.borrow();
        match locals.get(expression) {
            Some(distance) => {
                let last_env = {
                    let env_stack = self.environment_stack.borrow();
                    env_stack.last().unwrap().clone()
                };
                last_env.borrow().get_at(name, *distance)
            }
            None => self.globals.borrow().get(name),
        }
    }

    fn evaluate_lox_function(
        &self,
        token: &Token,
        closure: Rc<RefCell<Environment>>,
        params: &[Token],
        arguments: Vec<LoxValue>,
        block: &Block,
    ) -> InterpreterResult<LoxValue> {
        let mut function_env = Environment::new_enclosed(closure);

        if params.len() != arguments.len() {
            return interpreter_error!(
                InterpreterErrorType::WrongArity {
                    original: params.len(),
                    user: arguments.len()
                },
                token.clone()
            );
        }

        for (i, arg) in arguments.into_iter().enumerate() {
            function_env.define(params[i].lexeme().to_string(), arg);
        }

        let value = match self.execute_block(block, Rc::new(RefCell::new(function_env)), false)? {
            ControlFlow::Normal => LoxValue::Nil,
            ControlFlow::BreakLoop => LoxValue::Nil,
            ControlFlow::ContinueLoop => LoxValue::Nil,
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
            return interpreter_error!(
                InterpreterErrorType::WrongArity {
                    original: arity,
                    user: arguments.len()
                },
                token.clone()
            );
        }

        match func(arguments) {
            Ok(result) => Ok(result),
            Err(e) => interpreter_error!(InterpreterErrorType::Native(e), token.clone()),
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
            (op, expr) => interpreter_error!(
                InterpreterErrorType::WrongUnaryOperands(op.clone(), expr),
                token.clone()
            ),
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
                interpreter_error!(InterpreterErrorType::DivisionByZero, operator.clone())
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
            (t1, op, t2) => interpreter_error!(
                InterpreterErrorType::WrongBinaryOperands(t1, op.clone(), t2),
                operator.clone()
            ),
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
