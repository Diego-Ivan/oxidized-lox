pub(crate) use crate::interpreter::Interpreter;
use std::collections::HashMap;
use syntax::{Expression, Statement};

#[derive(thiserror::Error, Debug)]
pub enum ResolverError {
    #[error("Variable {0} cannot be read before it is initialized")]
    NotInitialized(String),
    #[error("Variable {0} is already declared in the current scope")]
    VariableAlreadyExists(String),
    #[error("Return statement has been used outside function")]
    ReturnNotInFunction,
    #[error("Invalid use of the this keyword in line {0}")]
    InvalidThis(usize),
}

enum FunctionType {
    None,
    Function,
    Method,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
}

pub struct Resolver<'i> {
    interpreter: &'i Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    function_type: FunctionType,
    class_type: ClassType,
}

impl<'i> Resolver<'i> {
    pub fn new(interpreter: &'i Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            function_type: FunctionType::None,
            class_type: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn resolve_statements(&mut self, statements: &[Statement]) -> Result<(), ResolverError> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, statement: &Statement) -> Result<(), ResolverError> {
        match statement {
            Statement::Block(block) => {
                self.begin_scope();
                self.resolve_statements(block)?;
                self.end_scope();
                Ok(())
            }

            Statement::VariableDeclaration { name, initializer } => {
                self.declare(name)?;

                if let Some(initializer) = initializer {
                    self.resolve_expression(initializer)?;
                }

                self.define(name);
                Ok(())
            }
            Statement::ClassDeclaration { name, methods } => {
                self.declare(name)?;
                self.define(name);

                let current_class = self.class_type;
                self.class_type = ClassType::Class;
                self.begin_scope();

                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(String::from("this"), true);
                }

                for method in methods {
                    self.function_type = FunctionType::Method;
                    self.resolve_function(&method.parameters, &method.body)?;
                }

                self.end_scope();
                self.class_type = current_class;

                Ok(())
            }
            Statement::Expression(expression) => self.resolve_expression(expression),
            Statement::Print(expression) => self.resolve_expression(expression),
            Statement::FunctionDeclaration(function) => {
                self.declare(&function.name)?;
                self.define(&function.name);

                self.resolve_function(&function.parameters, &function.body)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expression(condition)?;
                self.resolve_statement(then_branch)?;

                if let Some(else_branch) = else_branch {
                    self.resolve_statement(else_branch)?;
                }

                Ok(())
            }
            Statement::While { condition, body } => self
                .resolve_expression(condition)
                .and(self.resolve_statement(body)),
            Statement::For { .. } => todo!(),
            Statement::Return {
                keyword: _,
                expression,
            } => {
                if !matches!(
                    self.function_type,
                    FunctionType::Function | FunctionType::Method
                ) {
                    return Err(ResolverError::ReturnNotInFunction);
                }

                if let Some(expression) = expression {
                    self.resolve_expression(expression)?;
                }

                Ok(())
            }
            // TODO: Add support for checking that this is inside a loop
            Statement::Break { .. } => Ok(()),
            Statement::Continue { .. } => Ok(()),
        }
    }

    fn resolve_expression(&mut self, expr: &Expression) -> Result<(), ResolverError> {
        match expr {
            Expression::Var { name, token: _ } => {
                match self.scopes.last() {
                    Some(scope) if matches!(scope.get(name), Some(false)) => {
                        return Err(ResolverError::NotInitialized(String::from(name)));
                    }
                    Some(_) | None => self.resolve_local(expr, name),
                };

                Ok(())
            }
            Expression::This { keyword } => {
                if !matches!(self.class_type, ClassType::Class) {
                    return Err(ResolverError::InvalidThis(keyword.line()));
                }
                self.resolve_local(expr, keyword.lexeme());
                Ok(())
            }
            Expression::Binary { left, right, .. } => self
                .resolve_expression(left)
                .and(self.resolve_expression(right)),
            Expression::Grouping(expression) => self.resolve_expression(expression),
            Expression::Unary(_, expression) => self.resolve_expression(expression),
            Expression::Assignment {
                name,
                value,
                token: _,
            } => {
                self.resolve_expression(value)?;
                self.resolve_local(expr, name);

                Ok(())
            }
            // Logical Expressions
            Expression::Or { left, right } | Expression::And { left, right } => self
                .resolve_expression(left)
                .and(self.resolve_expression(right)),
            Expression::Call { callee, args, .. } => {
                self.resolve_expression(callee)?;

                for arg in args {
                    self.resolve_expression(arg)?;
                }

                Ok(())
            }
            Expression::Get { expression, .. } => self.resolve_expression(expression),
            Expression::Set { object, value, .. } => self
                .resolve_expression(object)
                .and(self.resolve_expression(value)),
            Expression::True
            | Expression::False
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::Nil => Ok(()),
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &[syntax::Token],
        body: &[Statement],
    ) -> Result<(), ResolverError> {
        self.function_type = FunctionType::Function;
        self.begin_scope();

        for param in parameters {
            self.declare(param.lexeme())?;
            self.define(param.lexeme());
        }

        self.resolve_statements(body)?;

        self.end_scope();
        self.function_type = FunctionType::None;

        Ok(())
    }

    fn resolve_local(&self, expr: &Expression, name: &str) {
        for (idx, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.interpreter.resolve(expr, idx);
                return;
            }
        }
    }

    fn define(&mut self, name: &str) {
        let scope = match self.scopes.last_mut() {
            Some(scope) => scope,
            None => return,
        };

        scope.insert(String::from(name), true);
    }

    fn declare(&mut self, name: &str) -> Result<(), ResolverError> {
        let scope = match self.scopes.last_mut() {
            Some(scope) => scope,
            None => return Ok(()),
        };

        if scope.contains_key(name) {
            return Err(ResolverError::VariableAlreadyExists(String::from(name)));
        }

        scope.insert(String::from(name), false);

        Ok(())
    }
}
